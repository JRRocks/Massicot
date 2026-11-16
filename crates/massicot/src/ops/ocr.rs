//! `ocr`: makes scanned pages searchable by adding the recognised words as invisible text, and
//! `texte`: the text of each page (text layer, or OCR).

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use pdfcraft_cos::{Dict, Document, ObjRef, Object, Stream};
use pdfcraft_ocr::{PlacedWord, TextPlan, Word};
use pdfcraft_render::PageInfo;
use serde::Serialize;

use crate::erreur::{Erreur, Resultat};
use crate::lecture::{Lecteur, LecturePage, ModeOcr, MoteurOcr, Mot, Source};
use crate::ops::sortie::{Ecrit, ecrire_pdf, verifier_cible};
use crate::pdf;

#[derive(Debug, Serialize)]
pub struct RapportOcr {
    #[serde(flatten)]
    pub fichier: Ecrit,
    /// The engine used ("integre" or "tesseract"), when at least one page was read.
    pub moteur: Option<String>,
    pub pages_reconnues: Vec<PageReconnue>,
    /// Pages left as they were because they already had text (1-based).
    pub pages_deja_texte: Vec<usize>,
}

#[derive(Debug, Serialize)]
pub struct PageReconnue {
    pub page: usize,
    pub mots: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confiance: Option<f32>,
}

impl RapportOcr {
    pub fn texte(&self) -> String {
        let mut s = self.fichier.texte();
        let n = self.pages_reconnues.len();
        let moteur = match self.moteur.as_deref() {
            Some("tesseract") => " (Tesseract)",
            Some(_) => " (moteur intégré)",
            None => "",
        };
        let _ = writeln!(s, "Pages reconnues{moteur} : {n}");
        for p in &self.pages_reconnues {
            let conf = p.confiance.map(|c| format!(", confiance {c:.0} %")).unwrap_or_default();
            let _ = writeln!(s, "  page {} : {} mots{conf}", p.page, p.mots);
        }
        if !self.pages_deja_texte.is_empty() {
            let _ = writeln!(s, "Pages qui avaient déjà du texte, laissées telles quelles : {}", pdf::resume_pages(&zero(&self.pages_deja_texte)));
        }
        s
    }
}

fn zero(pages: &[usize]) -> Vec<usize> {
    pages.iter().map(|p| p - 1).collect()
}

/// OCR of the pages of `entree` that have no text (or of every page with `toutes`), written to
/// `sortie` with the words as invisible, searchable text over the page picture.
pub fn ocr(entree: &Path, sortie: &Path, toutes: bool, moteur: MoteurOcr, dpi: f32, ecraser: bool) -> Resultat<RapportOcr> {
    verifier_cible(sortie, ecraser)?;
    let octets = std::fs::read(entree).map_err(|e| Erreur::fichier("lire", entree, &e))?;
    let mut doc = pdf::ouvrir_octets(entree, octets.clone())?;
    let mut lecteur = Lecteur::depuis_octets(entree, Arc::new(octets), moteur)?;
    lecteur.dpi = dpi;
    let pages = pdfcraft_organize::pages(&doc).map_err(|e| Erreur::moteur("la lecture des pages", e))?;
    let mut reconnues = Vec::new();
    let mut deja = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        if !toutes && lecteur.couche_texte(i)?.a_du_texte() {
            deja.push(i + 1);
            continue;
        }
        let lu = lecteur.ocr(i)?;
        ajouter_texte_invisible(&mut doc, page.obj, lecteur.info_page(i), &lu.mots)?;
        reconnues.push(PageReconnue { page: i + 1, mots: lu.mots.len(), confiance: lu.confiance });
    }
    let fichier = ecrire_pdf(&doc, sortie, ecraser)?;
    Ok(RapportOcr { fichier, moteur: lecteur.moteur_utilise().map(str::to_string), pages_reconnues: reconnues, pages_deja_texte: deja })
}

/// Adds `mots` (boxes in displayed-page points) to the page as invisible text (render mode 3),
/// the way PdfCraft's OCR does: Helvetica stretched to each word's box, in a marked `/OCR` block.
pub fn ajouter_texte_invisible(doc: &mut Document, page: ObjRef, info: &PageInfo, mots: &[Mot]) -> Resultat<()> {
    let places: Vec<PlacedWord> = mots
        .iter()
        .filter(|m| {
            let (l, h) = (m.rect[2] - m.rect[0], m.rect[3] - m.rect[1]);
            l > 0.0 && h > 0.0 && l.is_finite() && h.is_finite() && !m.texte.trim().is_empty()
        })
        .map(|m| {
            let w = Word::new(m.texte.clone(), m.rect, "massicot");
            PlacedWord::place(&w, |x, y| {
                let [u, v] = info.view_to_user(x, y);
                [u as f64, v as f64]
            })
        })
        .collect();
    if places.is_empty() {
        return Ok(());
    }
    let contenu = pdfcraft_ocr::text_layer(&places, &TextPlan::default());
    let echec = |e| Erreur::moteur("l'ajout du texte invisible", e);

    // The font the layer uses, /PCHelv, in the page's own copy of its resources.
    let mut ressources = ressources_heritees(doc, page);
    let mut polices = ressources.get(b"Font").and_then(|f| doc.dict(f)).unwrap_or_default();
    let mut helv = Dict::new();
    for (k, v) in [("Type", "Font"), ("Subtype", "Type1"), ("BaseFont", "Helvetica"), ("Encoding", "WinAnsiEncoding")] {
        helv.set(k.as_bytes().to_vec(), Object::name(v));
    }
    polices.set(b"PCHelv".to_vec(), Object::Dict(helv));
    ressources.set(b"Font".to_vec(), Object::Dict(polices));
    doc.update_dict(page, |d| d.set(b"Resources".to_vec(), Object::Dict(ressources))).map_err(echec)?;

    // The existing content is wrapped in q … Q (closing what it leaves open), then the layer.
    let actuel = doc.get(page).as_dict().and_then(|d| d.get(b"Contents").cloned());
    let mut liste = match actuel {
        None => Vec::new(),
        Some(Object::Ref(r)) => match doc.get(r).as_array() {
            Some(a) => a.clone(),
            None => vec![Object::Ref(r)],
        },
        Some(Object::Array(a)) => a,
        Some(autre) => vec![Object::Ref(doc.add(autre))],
    };
    if !liste.is_empty() {
        let (q_ouverts, marques_ouvertes) = ouverts(doc, &liste);
        let mut fin = String::new();
        for _ in 0..marques_ouvertes {
            fin.push_str("\nEMC");
        }
        for _ in 0..q_ouverts {
            fin.push_str("\nQ");
        }
        fin.push_str("\nQ\n");
        let debut = doc.add(Object::Stream(Stream::from_raw(Dict::new(), b"q\n".to_vec())));
        let fin = doc.add(Object::Stream(Stream::from_raw(Dict::new(), fin.into_bytes())));
        liste.insert(0, Object::Ref(debut));
        liste.push(Object::Ref(fin));
    }
    let mut marque = Dict::new();
    marque.set(b"PCMark".to_vec(), Object::name("OCR"));
    liste.push(Object::Ref(doc.add(Object::Stream(Stream::flate(marque, &contenu)))));
    doc.update_dict(page, |d| d.set(b"Contents".to_vec(), Object::Array(liste))).map_err(echec)?;
    Ok(())
}

/// The page's /Resources, inherited from the page tree when the page has none of its own.
fn ressources_heritees(doc: &Document, mut noeud: ObjRef) -> Dict {
    for _ in 0..64 {
        let o = doc.get(noeud);
        let Some(d) = o.as_dict() else { break };
        if let Some(r) = d.get(b"Resources") {
            return doc.dict(r).unwrap_or_default();
        }
        match d.reference(b"Parent") {
            Some(p) => noeud = p,
            None => break,
        }
    }
    Dict::new()
}

/// How many `q` and marked-content blocks the page content leaves open.
fn ouverts(doc: &Document, flux: &[Object]) -> (usize, usize) {
    let mut octets = Vec::new();
    for f in flux {
        if let Object::Stream(s) = &*doc.resolve(f) {
            if let Ok(d) = s.decoded() {
                octets.extend_from_slice(&d);
                octets.push(b'\n');
            }
        }
    }
    let (mut q, mut m) = (0usize, 0usize);
    for op in pdfcraft_content::parse(&octets).ops {
        if op.is("q") {
            q += 1;
        } else if op.is("Q") {
            q = q.saturating_sub(1);
        } else if op.is("BMC") || op.is("BDC") {
            m += 1;
        } else if op.is("EMC") {
            m = m.saturating_sub(1);
        }
    }
    (q, m)
}

/// The text of the selected pages.
#[derive(Debug, Serialize)]
pub struct Textes {
    pub fichier: String,
    pub pages: Vec<LecturePage>,
}

impl Textes {
    pub fn texte(&self) -> String {
        let mut s = String::new();
        for p in &self.pages {
            let source = match &p.source {
                Source::CoucheTexte => "couche texte".to_string(),
                Source::Ocr { moteur } if moteur == "tesseract" => "OCR Tesseract".to_string(),
                Source::Ocr { .. } => "OCR intégré".to_string(),
                Source::Aucune => "aucun texte".to_string(),
            };
            let _ = writeln!(s, "--- Page {} ({source}) ---", p.page);
            let t = p.texte.trim_end();
            if !t.is_empty() {
                let _ = writeln!(s, "{t}");
            }
        }
        s
    }
}

/// The text of each selected page (all by default), from its text layer or by OCR.
pub fn texte(entree: &Path, pages: Option<&str>, mode: ModeOcr, moteur: MoteurOcr) -> Resultat<Textes> {
    let mut lecteur = Lecteur::ouvrir(entree, moteur)?;
    let n = lecteur.nombre_de_pages();
    let indices = match pages {
        Some(p) => pdf::pages(p, n)?,
        None => (0..n).collect(),
    };
    let pages = indices.into_iter().map(|i| lecteur.lire(i, mode)).collect::<Resultat<Vec<_>>>()?;
    Ok(Textes { fichier: entree.display().to_string(), pages })
}
