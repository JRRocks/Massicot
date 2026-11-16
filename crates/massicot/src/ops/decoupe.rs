//! `decouper`: one PDF into several, every N pages, before given pages, at the bookmarks of a
//! chosen level, or by size.

use std::fmt::Write as _;
use std::ops::Range;
use std::path::Path;

use pdfcraft_cos::Document;
use pdfcraft_organize::SplitBy;
use serde::Serialize;

use crate::erreur::{Erreur, Resultat};
use crate::ops::sortie::{avec_extension_pdf, base_du_nom, dedoublonner, ecrire_lot, nom_sur};
use crate::pdf;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Every `n` pages.
    ToutesLes(usize),
    /// Before each of these pages (a selection as the user writes it: "4,9").
    Avant(String),
    /// At each bookmark of this level (1 = top level).
    Signets(usize),
    /// Parts of at most this many bytes (as far as single pages allow).
    Taille(u64),
}

#[derive(Debug, Serialize)]
pub struct Decoupe {
    pub fichiers: Vec<Partie>,
    /// Points to check (a page larger than the size limit…).
    pub alertes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Partie {
    pub fichier: String,
    /// 1-based pages of the source, e.g. "4-8".
    pub pages: String,
    pub nombre_pages: usize,
    pub taille_octets: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signet: Option<String>,
}

impl Decoupe {
    pub fn texte(&self) -> String {
        let mut s = String::new();
        let n = self.fichiers.len();
        let _ = writeln!(s, "{n} fichier{} écrit{} :", if n > 1 { "s" } else { "" }, if n > 1 { "s" } else { "" });
        for p in &self.fichiers {
            let mot = if p.nombre_pages > 1 { "pages" } else { "page" };
            let _ = writeln!(s, "  {} : {mot} {} ({})", p.fichier, p.pages, super::taille_lisible(p.taille_octets));
        }
        for a in &self.alertes {
            let _ = writeln!(s, "Attention : {a}");
        }
        s
    }
}

/// Splits `entree` into `dossier`. Part names start with `prefixe` (the file's name by default).
pub fn decouper(entree: &Path, mode: &Mode, dossier: &Path, prefixe: Option<&str>, ecraser: bool) -> Resultat<Decoupe> {
    let doc = pdf::ouvrir(entree)?;
    let n = pdf::nombre_de_pages(&doc)?;
    let base = nom_sur(prefixe.unwrap_or(&base_du_nom(entree)));
    let mut alertes = Vec::new();
    let (plages, titres): (Vec<Range<usize>>, Vec<Option<String>>) = match mode {
        Mode::ToutesLes(0) => return Err(Erreur::new("le nombre de pages par fichier doit être au moins 1")),
        Mode::ToutesLes(k) => {
            let p = pdfcraft_organize::split_ranges(n, &SplitBy::PageCount(*k));
            let t = vec![None; p.len()];
            (p, t)
        }
        Mode::Avant(selection) => {
            let coupes = pdf::pages(selection, n)?;
            let p = pdfcraft_organize::split_ranges(n, &SplitBy::Before(coupes));
            let t = vec![None; p.len()];
            (p, t)
        }
        Mode::Signets(niveau) => plages_par_signets(&doc, n, *niveau)?,
        Mode::Taille(limite) => {
            let p = plages_par_taille(&doc, n, *limite, &mut alertes)?;
            let t = vec![None; p.len()];
            (p, t)
        }
    };
    let largeur = plages.len().to_string().len().max(2);
    let mut noms: Vec<String> = plages
        .iter()
        .zip(&titres)
        .enumerate()
        .map(|(k, (_, titre))| match titre {
            Some(t) => avec_extension_pdf(format!("{:0largeur$}-{}", k + 1, nom_sur(t))),
            None => avec_extension_pdf(format!("{base}-{:0largeur$}", k + 1)),
        })
        .collect();
    dedoublonner(&mut noms);
    let mut docs = Vec::with_capacity(plages.len());
    for (nom, plage) in noms.iter().zip(&plages) {
        let pages: Vec<usize> = plage.clone().collect();
        let part = pdfcraft_organize::extract_pages(&doc, &pages).map_err(|e| Erreur::moteur("la découpe", e))?;
        docs.push((nom.clone(), part));
    }
    let ecrits = ecrire_lot(&docs, dossier, ecraser)?;
    let fichiers = ecrits
        .into_iter()
        .zip(plages)
        .zip(titres)
        .map(|((e, plage), signet)| Partie {
            fichier: e.fichier,
            pages: pdf::resume_pages(&plage.collect::<Vec<_>>()),
            nombre_pages: e.pages,
            taille_octets: e.taille_octets,
            signet,
        })
        .collect();
    Ok(Decoupe { fichiers, alertes })
}

/// Parts starting at each bookmark of `niveau`; pages before the first one form a part of their own.
fn plages_par_signets(doc: &Document, n: usize, niveau: usize) -> Resultat<(Vec<Range<usize>>, Vec<Option<String>>)> {
    if niveau == 0 {
        return Err(Erreur::new("le niveau de signet commence à 1 (les signets principaux)"));
    }
    let mut debuts: Vec<(usize, String)> = pdfcraft_organize::bookmarks(doc)
        .into_iter()
        .filter(|s| s.path.len() == niveau)
        .filter_map(|s| s.page.map(|p| (p, s.title)))
        .collect();
    if debuts.is_empty() {
        return Err(Erreur::new(format!("aucun signet de niveau {niveau} ne mène à une page de ce document")));
    }
    debuts.sort_by_key(|(p, _)| *p);
    debuts.dedup_by_key(|(p, _)| *p);
    let coupes: Vec<usize> = debuts.iter().map(|(p, _)| *p).collect();
    let plages = pdfcraft_organize::split_ranges(n, &SplitBy::Before(coupes));
    let titres = plages
        .iter()
        .map(|r| Some(debuts.iter().find(|(p, _)| *p == r.start).map(|(_, t)| t.clone()).unwrap_or_else(|| "début".into())))
        .collect();
    Ok((plages, titres))
}

fn taille_de(doc: &Document, pages: &[usize]) -> Resultat<u64> {
    let part = pdfcraft_organize::extract_pages(doc, pages).map_err(|e| Erreur::moteur("la mesure des pages", e))?;
    Ok(pdf::octets(&part)?.len() as u64)
}

/// Consecutive runs of pages whose file stays under `limite` bytes: an estimate from each page's
/// own size, then checked on the real part (a part still too large is cut in two, again and again).
fn plages_par_taille(doc: &Document, n: usize, limite: u64, alertes: &mut Vec<String>) -> Resultat<Vec<Range<usize>>> {
    if limite == 0 {
        return Err(Erreur::new("la taille maximale doit être supérieure à zéro"));
    }
    let mut seules = Vec::with_capacity(n);
    for i in 0..n {
        seules.push(taille_de(doc, &[i])?);
    }
    let socle = seules.iter().copied().min().unwrap_or(0) / 4;
    let mut plages = Vec::new();
    let (mut debut, mut cumul) = (0, socle);
    for (i, t) in seules.iter().enumerate() {
        let sans_socle = t.saturating_sub(socle);
        if i > debut && cumul + sans_socle > limite {
            plages.push(debut..i);
            debut = i;
            cumul = socle;
        }
        cumul += sans_socle;
    }
    if debut < n {
        plages.push(debut..n);
    }
    let mut verifiees = Vec::new();
    let mut pile: Vec<Range<usize>> = plages.into_iter().rev().collect();
    while let Some(r) = pile.pop() {
        let taille = taille_de(doc, &r.clone().collect::<Vec<_>>())?;
        if taille > limite && r.len() > 1 {
            let milieu = r.start + r.len() / 2;
            pile.push(milieu..r.end);
            pile.push(r.start..milieu);
        } else {
            if taille > limite {
                alertes.push(format!(
                    "la page {} pèse à elle seule {}, plus que la taille demandée",
                    r.start + 1,
                    super::taille_lisible(taille)
                ));
            }
            verifiees.push(r);
        }
    }
    Ok(verifiees)
}

/// Reads a size such as `5Mo`, `500 ko`, `1,5 Mo`, `2MB` or `300000` (bytes). 1 ko = 1024 octets.
pub fn lire_taille(texte: &str) -> Result<u64, String> {
    let t = texte.trim().to_lowercase().replace(',', ".");
    let fin_nombre = t.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(t.len());
    let (nombre, unite) = t.split_at(fin_nombre);
    let valeur: f64 = nombre.parse().map_err(|_| format!("« {texte} » n'est pas une taille (exemples : 5Mo, 500ko)"))?;
    let facteur: f64 = match unite.trim() {
        "" | "o" | "octet" | "octets" | "b" => 1.0,
        "k" | "ko" | "kio" | "kb" | "kib" => 1024.0,
        "m" | "mo" | "mio" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "go" | "gio" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        autre => return Err(format!("unité « {autre} » inconnue (o, ko, Mo ou Go)")),
    };
    let octets = (valeur * facteur).round();
    if octets < 1.0 {
        return Err("la taille doit être supérieure à zéro".into());
    }
    Ok(octets as u64)
}

#[cfg(test)]
mod tests {
    use super::lire_taille;

    #[test]
    fn sizes_accept_french_and_english_units() {
        assert_eq!(lire_taille("5Mo"), Ok(5 * 1024 * 1024));
        assert_eq!(lire_taille("500 ko"), Ok(500 * 1024));
        assert_eq!(lire_taille("1,5 Mo"), Ok(1_572_864));
        assert_eq!(lire_taille("2MB"), Ok(2 * 1024 * 1024));
        assert_eq!(lire_taille("300000"), Ok(300_000));
        assert!(lire_taille("5 litres").is_err());
        assert!(lire_taille("Mo").is_err());
    }
}
