//! `decouper-regex`: cuts a batch of scanned documents at each page that carries a header found
//! by a regular expression (one PDF per block), with no AI involved. A dry run (the default)
//! reports the blocks; the real cut needs the token of that dry run.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::Regex;
use regex_syntax::hir::{Class, ClassUnicode, ClassUnicodeRange, Hir, HirKind};
use serde::Serialize;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::erreur::{Erreur, Resultat};
use crate::lecture::{Lecteur, LecturePage, ModeOcr, MoteurOcr, Mot, Source};
use crate::ops::sortie::{avec_extension_pdf, base_du_nom, dedoublonner, ecrire_lot, nom_sur, verifier_cible};
use crate::pdf;

/// What to do with the pages before the first header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum AvantPremier {
    /// A file of their own (the default), reported.
    APart,
    /// Joined to the first block.
    Premier,
    /// Left out (reported).
    Ignorer,
}

/// The settings of a cut. Everything that changes the result is part of the dry-run token.
#[derive(Debug, Clone, Serialize)]
pub struct Reglages {
    pub regex: Vec<String>,
    /// File name template: `{bloc}`, `{page}`, `{fichier}`, `{texte}`, and the regex groups
    /// (`{1}`, `{num}`…).
    pub modele: String,
    pub ocr: ModeOcr,
    pub moteur: MoteurOcr,
    pub dpi: u32,
    /// Only look at the top of the page: this percentage of its height.
    pub zone_haute: Option<u8>,
    pub respecter_casse: bool,
    pub respecter_accents: bool,
    pub avant_premier: AvantPremier,
}

impl Reglages {
    pub fn new(regex: Vec<String>) -> Self {
        Reglages {
            regex,
            modele: MODELE_PAR_DEFAUT.into(),
            ocr: ModeOcr::Auto,
            moteur: MoteurOcr::Auto,
            dpi: 300,
            zone_haute: None,
            respecter_casse: false,
            respecter_accents: false,
            avant_premier: AvantPremier::APart,
        }
    }
}

pub const MODELE_PAR_DEFAUT: &str = "{fichier}-{bloc}.pdf";
const NOM_AVANT_PREMIER: &str = "00-pages-avant-le-premier-en-tete.pdf";
/// Below this OCR confidence (0-100), a header is flagged.
const CONFIANCE_FAIBLE: f32 = 60.0;

#[derive(Debug, Serialize)]
pub struct Rapport {
    pub fichier_source: String,
    pub a_blanc: bool,
    /// Give this back with `--jeton` to cut for real (same file, same settings).
    pub jeton: String,
    pub reglages: Reglages,
    pub nombre_de_pages: usize,
    /// Pages read by OCR, and the engine used.
    pub pages_lues_par_ocr: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moteur_ocr: Option<String>,
    pub blocs: Vec<Bloc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pages_ignorees: Vec<usize>,
    pub alertes: Vec<String>,
    /// Where the files and this report were written (real cut only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dossier: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Bloc {
    pub numero: usize,
    pub fichier: String,
    /// 1-based pages of the source, e.g. "4-7".
    pub pages: String,
    pub premiere_page: usize,
    pub derniere_page: usize,
    pub nombre_pages: usize,
    /// The header that opens the block (absent for pages before the first header).
    pub en_tete: Option<EnTete>,
    /// The beginning of the first page's text.
    pub extrait: String,
    pub alertes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taille_octets: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnTete {
    pub page: usize,
    /// Which regex matched (1-based, in the order given).
    pub regex: usize,
    /// The text the regex matched, as the page reads (accents kept).
    pub texte_trouve: String,
    pub groupes: BTreeMap<String, String>,
    pub source: Source,
    /// Mean OCR confidence of the page (0-100), when the engine gives one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confiance: Option<f32>,
}

/// Text normalised for matching, with the way back to the original text.
struct Normalise {
    texte: String,
    /// For each byte of `texte`: the byte range of the original character it comes from.
    origine: Vec<(usize, usize)>,
}

/// Collapses whitespace runs into one space and, unless `accents`, drops accents (NFKD without
/// combining marks: « PROCÈS » → « PROCES », « ﬁ » → « fi »).
fn normaliser(source: &str, accents: bool) -> Normalise {
    let mut texte = String::with_capacity(source.len());
    let mut origine = Vec::with_capacity(source.len());
    let mut espace = false;
    for (i, c) in source.char_indices() {
        let fin = i + c.len_utf8();
        if c.is_whitespace() {
            if !texte.is_empty() && !espace {
                texte.push(' ');
                origine.push((i, fin));
            }
            espace = true;
            continue;
        }
        espace = false;
        let mut pousser = |d: char| {
            for _ in 0..d.len_utf8() {
                origine.push((i, fin));
            }
            texte.push(d);
        };
        if accents {
            pousser(c);
        } else {
            for d in c.to_string().nfkd().filter(|d| !is_combining_mark(*d)) {
                pousser(d);
            }
        }
    }
    if texte.ends_with(' ') {
        texte.pop();
        origine.pop();
    }
    Normalise { texte, origine }
}

impl Normalise {
    /// The original text behind a byte range of the normalised text.
    fn original<'a>(&self, source: &'a str, debut: usize, fin: usize) -> &'a str {
        if debut >= fin || fin > self.origine.len() {
            return "";
        }
        &source[self.origine[debut].0..self.origine[fin - 1].1]
    }
}

/// A user's regex, compiled for clean text and, for pages read by the built-in OCR engine
/// (which writes « ? » for every letter it does not know, such as accented ones), a tolerant
/// variant where each letter or symbol may also be « ? ».
struct Motif {
    exact: Regex,
    tolerant: Regex,
}

fn compiler(source: &str, reglages: &Reglages) -> Resultat<Motif> {
    let mut motif = if reglages.respecter_accents { source.to_string() } else { replier_accents(source) };
    if !reglages.respecter_casse {
        motif = format!("(?i:{motif})");
    }
    let invalide = |e: &dyn std::fmt::Display| Erreur::new(format!("expression régulière « {source} » invalide : {e}"));
    let exact = Regex::new(&motif).map_err(|e| invalide(&e))?;
    let hir = regex_syntax::ParserBuilder::new().build().parse(&motif).map_err(|e| invalide(&e))?;
    let tolerant = Regex::new(&tolerer(hir).to_string()).map_err(|e| invalide(&e))?;
    Ok(Motif { exact, tolerant })
}

/// Drops accents from the non-ASCII characters of a pattern (ASCII is left alone, so the regex
/// syntax is never changed).
fn replier_accents(motif: &str) -> String {
    motif
        .chars()
        .flat_map(|c| if c.is_ascii() { vec![c] } else { c.to_string().nfkd().filter(|d| !is_combining_mark(*d)).collect() })
        .collect()
}

/// Lets every letter or non-ASCII symbol of `hir` also match « ? ».
fn tolerer(hir: Hir) -> Hir {
    let ajouter = |c: char| -> Hir {
        if c.is_alphabetic() || !c.is_ascii() {
            let mut classe = ClassUnicode::new([ClassUnicodeRange::new(c, c)]);
            classe.push(ClassUnicodeRange::new('?', '?'));
            Hir::class(Class::Unicode(classe))
        } else {
            Hir::literal(c.to_string().into_bytes())
        }
    };
    match hir.into_kind() {
        HirKind::Literal(lit) => match std::str::from_utf8(&lit.0) {
            Ok(s) => Hir::concat(s.chars().map(ajouter).collect()),
            Err(_) => Hir::literal(lit.0),
        },
        HirKind::Class(Class::Unicode(mut classe)) => {
            if classe.iter().any(|r| (r.start()..=r.end()).any(|c| c.is_alphabetic())) {
                classe.push(ClassUnicodeRange::new('?', '?'));
            }
            Hir::class(Class::Unicode(classe))
        }
        HirKind::Repetition(mut r) => {
            r.sub = Box::new(tolerer(*r.sub));
            Hir::repetition(r)
        }
        HirKind::Capture(mut c) => {
            c.sub = Box::new(tolerer(*c.sub));
            Hir::capture(c)
        }
        HirKind::Concat(subs) => Hir::concat(subs.into_iter().map(tolerer).collect()),
        HirKind::Alternation(subs) => Hir::alternation(subs.into_iter().map(tolerer).collect()),
        HirKind::Empty => Hir::empty(),
        HirKind::Class(c) => Hir::class(c),
        HirKind::Look(l) => Hir::look(l),
    }
}

/// The text a header is looked for in: the words of the page (or of its top band), line by line.
fn texte_cherche(page: &LecturePage, zone_haute: Option<u8>) -> String {
    let limite = zone_haute.map(|z| page.hauteur * z as f32 / 100.0);
    let mots: Vec<&Mot> = page.mots.iter().filter(|m| limite.is_none_or(|l| (m.rect[1] + m.rect[3]) / 2.0 <= l)).collect();
    let mut s = String::new();
    let mut ligne = None;
    for m in mots {
        if !s.is_empty() {
            s.push(if ligne == Some(m.ligne) { ' ' } else { '\n' });
        }
        s.push_str(&m.texte);
        ligne = Some(m.ligne);
    }
    s
}

fn extrait(page: &LecturePage) -> String {
    let t = normaliser(&page.texte, true).texte;
    match t.char_indices().nth(240) {
        Some((i, _)) => format!("{}…", &t[..i]),
        None => t,
    }
}

/// The header of a page, if one of the regexes finds one.
fn chercher(page: &LecturePage, motifs: &[Motif], reglages: &Reglages) -> Option<EnTete> {
    let source = texte_cherche(page, reglages.zone_haute);
    let normal = normaliser(&source, reglages.respecter_accents);
    let tolerant = matches!(&page.source, Source::Ocr { moteur } if moteur == "integre");
    for (k, m) in motifs.iter().enumerate() {
        let re = if tolerant { &m.tolerant } else { &m.exact };
        let Some(c) = re.captures(&normal.texte) else { continue };
        let entier = c.get(0)?;
        let mut groupes = BTreeMap::new();
        for (i, nom) in re.capture_names().enumerate().skip(1) {
            if let Some(g) = c.get(i) {
                let valeur = normal.original(&source, g.start(), g.end()).to_string();
                groupes.insert(i.to_string(), valeur.clone());
                if let Some(nom) = nom {
                    groupes.insert(nom.to_string(), valeur);
                }
            }
        }
        return Some(EnTete {
            page: page.page,
            regex: k + 1,
            texte_trouve: normal.original(&source, entier.start(), entier.end()).to_string(),
            groupes,
            source: page.source.clone(),
            confiance: page.confiance,
        });
    }
    None
}

/// The variables a template may use: built-ins and the groups of every regex.
fn verifier_modele(modele: &str, motifs: &[Motif]) -> Resultat<()> {
    let mut connus: Vec<String> = ["bloc", "page", "fichier", "texte"].iter().map(|s| s.to_string()).collect();
    for m in motifs {
        for (i, nom) in m.exact.capture_names().enumerate().skip(1) {
            connus.push(i.to_string());
            if let Some(nom) = nom {
                connus.push(nom.to_string());
            }
        }
    }
    let mut reste = modele;
    while let Some(debut) = reste.find('{') {
        let Some(fin) = reste[debut..].find('}') else {
            return Err(Erreur::new(format!("modèle de nom « {modele} » : accolade « {{ » non fermée")));
        };
        let nom = &reste[debut + 1..debut + fin];
        if !connus.iter().any(|c| c == nom) {
            return Err(Erreur::new(format!(
                "modèle de nom « {modele} » : variable {{{nom}}} inconnue (disponibles : {})",
                connus.iter().map(|c| format!("{{{c}}}")).collect::<Vec<_>>().join(", ")
            )));
        }
        reste = &reste[debut + fin + 1..];
    }
    Ok(())
}

fn remplir(modele: &str, valeurs: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let mut reste = modele;
    while let Some(debut) = reste.find('{') {
        out.push_str(&reste[..debut]);
        let fin = reste[debut..].find('}').map_or(reste.len(), |f| debut + f);
        let nom = &reste[debut + 1..fin.min(reste.len())];
        out.push_str(valeurs.get(nom).map(String::as_str).unwrap_or(""));
        reste = &reste[(fin + 1).min(reste.len())..];
    }
    out.push_str(reste);
    out
}

/// The dry-run token: the file's bytes and every setting that changes the result.
fn jeton(octets: &[u8], reglages: &Reglages) -> String {
    let mut h = Sha256::new();
    h.update(octets);
    h.update([0]);
    h.update(serde_json::to_vec(reglages).unwrap_or_default());
    h.finalize().iter().take(6).map(|b| format!("{b:02x}")).collect()
}

/// Analyses `entree` and, when `couper` holds the dry run's token, writes one PDF per block into
/// `dossier` with the report beside them (`rapport-decoupe.json`).
pub fn decouper_regex(entree: &Path, dossier: &Path, reglages: &Reglages, couper: Option<&str>, ecraser: bool) -> Resultat<Rapport> {
    if reglages.regex.is_empty() {
        return Err(Erreur::new("indiquez au moins une expression régulière (--regex)"));
    }
    if let Some(z) = reglages.zone_haute {
        if z == 0 || z > 100 {
            return Err(Erreur::new("la zone haute est un pourcentage entre 1 et 100"));
        }
    }
    let motifs = reglages.regex.iter().map(|r| compiler(r, reglages)).collect::<Resultat<Vec<_>>>()?;
    verifier_modele(&reglages.modele, &motifs)?;
    let octets = std::fs::read(entree).map_err(|e| Erreur::fichier("lire", entree, &e))?;
    let jeton_attendu = jeton(&octets, reglages);
    if let Some(donne) = couper {
        if donne.trim() != jeton_attendu {
            return Err(Erreur::new(
                "jeton inconnu : le fichier ou les réglages ont changé depuis le mode à blanc, ou le jeton est faux ; \
                 relancez le mode à blanc et vérifiez son rapport avant de couper",
            ));
        }
    }
    let mut lecteur = Lecteur::depuis_octets(entree, Arc::new(octets.clone()), reglages.moteur)?;
    lecteur.dpi = reglages.dpi as f32;
    let n = lecteur.nombre_de_pages();
    if n == 0 {
        return Err(Erreur::new(format!("« {} » ne contient aucune page lisible", entree.display())));
    }
    let mut lectures = Vec::with_capacity(n);
    let mut en_tetes = Vec::with_capacity(n);
    for i in 0..n {
        let page = lecteur.lire(i, reglages.ocr)?;
        en_tetes.push(chercher(&page, &motifs, reglages));
        lectures.push(page);
    }
    let par_ocr = lectures.iter().filter(|p| matches!(p.source, Source::Ocr { .. })).count();

    // Blocks: each header page opens one, which runs until the next header.
    let debuts: Vec<usize> = (0..n).filter(|i| en_tetes[*i].is_some()).collect();
    let mut alertes = Vec::new();
    let mut plages: Vec<(std::ops::Range<usize>, Option<EnTete>)> = Vec::new();
    let mut ignorees = Vec::new();
    if debuts.is_empty() {
        alertes.push("aucun en-tête trouvé : vérifiez l'expression régulière (et la zone haute, si elle est réglée)".into());
    } else {
        for (k, d) in debuts.iter().enumerate() {
            let fin = debuts.get(k + 1).copied().unwrap_or(n);
            plages.push((*d..fin, en_tetes[*d].clone()));
        }
        let premier = debuts[0];
        if premier > 0 {
            let avant = format!("page{} 1{}", if premier > 1 { "s" } else { "" }, if premier > 1 { format!("-{premier}") } else { String::new() });
            match reglages.avant_premier {
                AvantPremier::APart => {
                    plages.insert(0, (0..premier, None));
                    alertes.push(format!("{avant} avant le premier en-tête : écrite(s) à part dans « {NOM_AVANT_PREMIER} »"));
                }
                AvantPremier::Premier => {
                    plages[0].0.start = 0;
                    alertes.push(format!("{avant} avant le premier en-tête : jointe(s) au premier bloc"));
                }
                AvantPremier::Ignorer => {
                    ignorees = (1..=premier).collect();
                    alertes.push(format!("{avant} avant le premier en-tête : laissée(s) de côté"));
                }
            }
        }
    }

    // Names from the template, made safe, then numbered when they repeat.
    let base = base_du_nom(entree);
    let numerotes = plages.iter().filter(|(_, e)| e.is_some()).count();
    let largeur = numerotes.to_string().len().max(2);
    let mut noms = Vec::with_capacity(plages.len());
    let mut numero = 0;
    for (plage, en_tete) in &plages {
        let nom = match en_tete {
            None => NOM_AVANT_PREMIER.to_string(),
            Some(e) => {
                numero += 1;
                let mut valeurs = e.groupes.clone();
                valeurs.insert("bloc".into(), format!("{numero:0largeur$}"));
                valeurs.insert("page".into(), (plage.start + 1).to_string());
                valeurs.insert("fichier".into(), base.clone());
                valeurs.insert("texte".into(), e.texte_trouve.clone());
                let rempli = nom_sur(&remplir(&reglages.modele, &valeurs));
                if rempli.trim_matches(|c: char| !c.is_alphanumeric()).is_empty() || rempli.eq_ignore_ascii_case(".pdf") {
                    avec_extension_pdf(format!("{base}-{numero:0largeur$}"))
                } else {
                    avec_extension_pdf(rempli)
                }
            }
        };
        noms.push(nom);
    }
    let avant = noms.clone();
    let renommes = dedoublonner(&mut noms);

    let mut blocs = Vec::with_capacity(plages.len());
    for (k, ((plage, en_tete), nom)) in plages.iter().zip(&noms).enumerate() {
        let mut a = Vec::new();
        if renommes[k] {
            a.push(format!("nom « {} » déjà pris : ce bloc sera écrit sous « {nom} »", avant[k]));
        }
        if let Some(e) = en_tete {
            if let Some(c) = e.confiance.filter(|c| *c < CONFIANCE_FAIBLE) {
                a.push(format!("en-tête lu par OCR avec une confiance faible ({c:.0} %) : à vérifier"));
            }
            if matches!(&e.source, Source::Ocr { moteur } if moteur == "integre") && e.texte_trouve.contains('?') {
                a.push("en-tête lu par le moteur OCR intégré, qui écrit « ? » pour les lettres accentuées : à vérifier".into());
            }
        }
        for i in plage.clone() {
            if !lectures[i].a_du_texte() {
                a.push(format!("page {} : aucun texte (page blanche, ou illisible)", i + 1));
            }
        }
        let pages: Vec<usize> = plage.clone().collect();
        blocs.push(Bloc {
            numero: k + 1,
            fichier: nom.clone(),
            pages: pdf::resume_pages(&pages),
            premiere_page: plage.start + 1,
            derniere_page: plage.end,
            nombre_pages: plage.len(),
            en_tete: en_tete.clone(),
            extrait: extrait(&lectures[plage.start]),
            alertes: a,
            taille_octets: None,
        });
    }
    if par_ocr > 0 && lecteur.moteur_utilise() == Some("integre") {
        alertes.push("OCR par le moteur intégré : il ne donne pas de confiance et lit les accents comme « ? »".into());
    }
    if couper.is_none() && !ecraser {
        for b in &blocs {
            let chemin = dossier.join(&b.fichier);
            if chemin.exists() {
                alertes.push(format!("« {} » existe déjà : la découpe demandera --ecraser", chemin.display()));
            }
        }
    }
    let mut rapport = Rapport {
        fichier_source: entree.display().to_string(),
        a_blanc: couper.is_none(),
        jeton: jeton_attendu,
        reglages: reglages.clone(),
        nombre_de_pages: n,
        pages_lues_par_ocr: par_ocr,
        moteur_ocr: lecteur.moteur_utilise().map(str::to_string),
        blocs,
        pages_ignorees: ignorees,
        alertes,
        dossier: None,
    };
    if couper.is_none() {
        return Ok(rapport);
    }
    if rapport.blocs.is_empty() {
        return Err(Erreur::new("aucun en-tête trouvé : rien n'est découpé"));
    }

    // The real cut: every target is checked before the first file is written.
    let chemin_rapport = dossier.join("rapport-decoupe.json");
    verifier_cible(&chemin_rapport, ecraser)?;
    let doc = pdf::ouvrir_octets(entree, octets)?;
    let mut docs = Vec::with_capacity(rapport.blocs.len());
    for b in &rapport.blocs {
        let pages: Vec<usize> = (b.premiere_page - 1..b.derniere_page).collect();
        let part = pdfcraft_organize::extract_pages(&doc, &pages).map_err(|e| Erreur::moteur("la découpe", e))?;
        docs.push((b.fichier.clone(), part));
    }
    let ecrits = ecrire_lot(&docs, dossier, ecraser)?;
    for (b, e) in rapport.blocs.iter_mut().zip(ecrits) {
        b.taille_octets = Some(e.taille_octets);
    }
    rapport.dossier = Some(dossier.display().to_string());
    let json = serde_json::to_vec_pretty(&rapport).map_err(|e| Erreur::moteur("l'écriture du rapport", e))?;
    pdf::ecrire(&chemin_rapport, &json)?;
    Ok(rapport)
}

/// Saves a report as JSON (dry run `--rapport`).
pub fn enregistrer_rapport(rapport: &Rapport, chemin: &Path, ecraser: bool) -> Resultat<PathBuf> {
    verifier_cible(chemin, ecraser)?;
    let json = serde_json::to_vec_pretty(rapport).map_err(|e| Erreur::moteur("l'écriture du rapport", e))?;
    pdf::ecrire(chemin, &json)?;
    Ok(chemin.to_path_buf())
}

impl Rapport {
    pub fn texte(&self) -> String {
        let mut s = String::new();
        if self.a_blanc {
            let _ = writeln!(s, "Mode à blanc : aucun fichier n'est écrit.");
        } else if let Some(d) = &self.dossier {
            let _ = writeln!(s, "Découpe faite dans « {d} » (rapport : rapport-decoupe.json).");
        }
        let ocr = match (self.pages_lues_par_ocr, self.moteur_ocr.as_deref()) {
            (0, _) => String::new(),
            (k, Some("tesseract")) => format!(", dont {k} par OCR (Tesseract)"),
            (k, _) => format!(", dont {k} par OCR (moteur intégré)"),
        };
        let _ = writeln!(s, "{} pages lues{ocr}.", self.nombre_de_pages);
        let _ = writeln!(s, "{} bloc{} :", self.blocs.len(), if self.blocs.len() > 1 { "s" } else { "" });
        for b in &self.blocs {
            let mot = if b.nombre_pages > 1 { "pages" } else { "page" };
            match &b.en_tete {
                Some(e) => {
                    let _ = writeln!(s, "  {}. {} : {mot} {} (en-tête page {} : « {} »)", b.numero, b.fichier, b.pages, e.page, e.texte_trouve);
                }
                None => {
                    let _ = writeln!(s, "  {}. {} : {mot} {} (sans en-tête)", b.numero, b.fichier, b.pages);
                }
            }
            for a in &b.alertes {
                let _ = writeln!(s, "     attention : {a}");
            }
        }
        for a in &self.alertes {
            let _ = writeln!(s, "Attention : {a}");
        }
        if self.a_blanc && !self.blocs.is_empty() {
            let _ = writeln!(s, "Pour couper : mêmes options, plus --couper {}", self.jeton);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalisation_drops_accents_and_collapses_spaces_but_keeps_the_original() {
        let src = "  PROCÈS-VERBAL\n\tN°  12 — Élection ";
        let n = normaliser(src, false);
        assert_eq!(n.texte, "PROCES-VERBAL N° 12 — Election");
        let debut = n.texte.find("Election").unwrap();
        assert_eq!(n.original(src, debut, debut + "Election".len()), "Élection");
        assert_eq!(normaliser("ﬁn  du", false).texte, "fin du");
        assert_eq!(normaliser("Été", true).texte, "Été");
    }

    #[test]
    fn regexes_ignore_case_and_accents_by_default() {
        let r = Reglages::new(vec![]);
        let m = compiler(r"PROCÈS-VERBAL N° (\d+)", &r).unwrap();
        assert!(m.exact.is_match("proces-verbal N° 12"));
        let strict = Reglages { respecter_casse: true, respecter_accents: true, ..Reglages::new(vec![]) };
        let m = compiler(r"PROCÈS-VERBAL N° (\d+)", &strict).unwrap();
        assert!(!m.exact.is_match("proces-verbal N° 12"));
        assert!(m.exact.is_match("PROCÈS-VERBAL N° 12"));
    }

    #[test]
    fn the_tolerant_variant_reads_through_unknown_letters() {
        let m = compiler(r"PROCÈS-VERBAL N° (?<num>\d+)", &Reglages::new(vec![])).unwrap();
        assert!(!m.exact.is_match("PROC?S-VERBAL N? 12"));
        let c = m.tolerant.captures("PROC?S-VERBAL N? 12").unwrap();
        assert_eq!(&c["num"], "12");
        // Digits stay digits: « ? » never stands for a number.
        assert!(!m.tolerant.is_match("PROCES-VERBAL N° ?"));
        let m = compiler(r"^D[ée]lib[ée]ration", &Reglages::new(vec![])).unwrap();
        assert!(m.tolerant.is_match("D?lib?ration n? 4"));
    }

    #[test]
    fn templates_check_their_variables_and_fill_them() {
        let m = vec![compiler(r"PV N° (?<num>\d+)", &Reglages::new(vec![])).unwrap()];
        assert!(verifier_modele("PV-{num}.pdf", &m).is_ok());
        assert!(verifier_modele("PV-{1}-{bloc}-{page}-{fichier}.pdf", &m).is_ok());
        let e = verifier_modele("PV-{numero}.pdf", &m).unwrap_err();
        assert!(e.message().contains("{numero} inconnue") && e.message().contains("{num}"), "{e}");
        let mut v = BTreeMap::new();
        v.insert("num".to_string(), "12".to_string());
        assert_eq!(remplir("PV-{num}.pdf", &v), "PV-12.pdf");
        assert_eq!(remplir("PV-{absent}.pdf", &v), "PV-.pdf");
    }
}
