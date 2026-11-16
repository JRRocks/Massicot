//! `infos`: what a PDF contains (pages, formats, metadata, bookmarks, protection).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::Serialize;

use crate::erreur::{Erreur, Resultat};
use crate::pdf;

#[derive(Debug, Serialize)]
pub struct Infos {
    pub fichier: String,
    pub taille_octets: u64,
    pub pages: usize,
    pub version_pdf: String,
    pub protege: bool,
    /// Title, author… (keys in French: titre, auteur, sujet, mots_cles, createur, producteur).
    pub metadonnees: BTreeMap<String, String>,
    pub signets: usize,
    pub niveaux_de_signets: usize,
    /// Runs of consecutive pages with the same visible size.
    pub formats: Vec<FormatPages>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FormatPages {
    /// 1-based pages, e.g. "1-12".
    pub pages: String,
    pub largeur_mm: f64,
    pub hauteur_mm: f64,
    /// "A4 portrait", "A3 paysage", or "210 × 297 mm" when the size has no usual name.
    pub format: String,
}

const META: [(&str, &str); 6] = [
    ("Title", "titre"),
    ("Author", "auteur"),
    ("Subject", "sujet"),
    ("Keywords", "mots_cles"),
    ("Creator", "createur"),
    ("Producer", "producteur"),
];

pub fn infos(chemin: &Path) -> Resultat<Infos> {
    let taille = std::fs::metadata(chemin).map_err(|e| Erreur::fichier("lire", chemin, &e))?.len();
    let doc = pdf::ouvrir(chemin)?;
    let pages = pdf::nombre_de_pages(&doc)?;
    let mut metadonnees = BTreeMap::new();
    for (cle, nom) in META {
        if let Some(v) = pdfcraft_organize::info(&doc, cle).filter(|v| !v.trim().is_empty()) {
            metadonnees.insert(nom.to_string(), v);
        }
    }
    let signets = pdfcraft_organize::bookmarks(&doc);
    let boites = pdfcraft_organize::page_boxes(&doc).map_err(|e| Erreur::moteur("la lecture des formats de page", e))?;
    let mut tailles = Vec::with_capacity(boites.len());
    for (i, b) in boites.iter().enumerate() {
        let crop = b[1];
        let (l, h) = ((crop[2] - crop[0]).abs(), (crop[3] - crop[1]).abs());
        let rotation = pdfcraft_organize::page_rotation(&doc, i).unwrap_or(0).rem_euclid(360);
        let (l, h) = if rotation == 90 || rotation == 270 { (h, l) } else { (l, h) };
        tailles.push((pt_vers_mm(l), pt_vers_mm(h)));
    }
    Ok(Infos {
        fichier: chemin.display().to_string(),
        taille_octets: taille,
        pages,
        version_pdf: doc.version().to_string(),
        protege: doc.security().is_some(),
        metadonnees,
        signets: signets.len(),
        niveaux_de_signets: signets.iter().map(|s| s.path.len()).max().unwrap_or(0),
        formats: regrouper_formats(&tailles),
    })
}

fn pt_vers_mm(pt: f64) -> f64 {
    (pt * 25.4 / 72.0 * 10.0).round() / 10.0
}

/// The usual name of a page size, within 2 mm.
pub fn nom_format(largeur: f64, hauteur: f64) -> String {
    const FORMATS: [(&str, f64, f64); 7] = [
        ("A3", 297.0, 420.0),
        ("A4", 210.0, 297.0),
        ("A5", 148.0, 210.0),
        ("A6", 105.0, 148.0),
        ("Lettre US", 215.9, 279.4),
        ("Légal US", 215.9, 355.6),
        ("Tabloïd", 279.4, 431.8),
    ];
    let (petit, grand) = if largeur <= hauteur { (largeur, hauteur) } else { (hauteur, largeur) };
    let orientation = if largeur <= hauteur { "portrait" } else { "paysage" };
    for (nom, l, h) in FORMATS {
        if (petit - l).abs() <= 2.0 && (grand - h).abs() <= 2.0 {
            return format!("{nom} {orientation}");
        }
    }
    format!("{} × {} mm", nombre_fr(largeur), nombre_fr(hauteur))
}

fn nombre_fr(x: f64) -> String {
    if (x - x.round()).abs() < 0.05 { format!("{}", x.round() as i64) } else { format!("{x:.1}").replace('.', ",") }
}

fn regrouper_formats(tailles: &[(f64, f64)]) -> Vec<FormatPages> {
    let mut out: Vec<FormatPages> = Vec::new();
    let mut debut = 0;
    for i in 0..tailles.len() {
        let fin_de_groupe = i + 1 == tailles.len() || tailles[i + 1] != tailles[debut];
        if fin_de_groupe {
            let (l, h) = tailles[debut];
            out.push(FormatPages {
                pages: pdf::resume_pages(&(debut..=i).collect::<Vec<_>>()),
                largeur_mm: l,
                hauteur_mm: h,
                format: nom_format(l, h),
            });
            debut = i + 1;
        }
    }
    out
}

impl Infos {
    pub fn texte(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "Fichier : {}", self.fichier);
        let _ = writeln!(s, "Taille : {}", super::taille_lisible(self.taille_octets));
        let _ = writeln!(s, "Pages : {}", self.pages);
        let _ = writeln!(s, "Version PDF : {}", self.version_pdf);
        let _ = writeln!(s, "Protégé : {}", if self.protege { "oui" } else { "non" });
        for (cle, nom) in [
            ("titre", "Titre"),
            ("auteur", "Auteur"),
            ("sujet", "Sujet"),
            ("mots_cles", "Mots-clés"),
            ("createur", "Créé avec"),
            ("producteur", "Produit par"),
        ] {
            if let Some(v) = self.metadonnees.get(cle) {
                let _ = writeln!(s, "{nom} : {v}");
            }
        }
        match self.signets {
            0 => {
                let _ = writeln!(s, "Signets : aucun");
            }
            n => {
                let niveaux = if self.niveaux_de_signets > 1 { "niveaux" } else { "niveau" };
                let _ = writeln!(s, "Signets : {n} ({} {niveaux})", self.niveaux_de_signets);
            }
        }
        if self.formats.len() == 1 {
            let _ = writeln!(s, "Format : {}", self.formats[0].format);
        } else {
            let _ = writeln!(s, "Formats :");
            for f in &self.formats {
                let _ = writeln!(s, "  pages {} : {}", f.pages, f.format);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usual_sizes_have_names() {
        assert_eq!(nom_format(210.0, 297.0), "A4 portrait");
        assert_eq!(nom_format(297.0, 210.0), "A4 paysage");
        assert_eq!(nom_format(215.9, 279.4), "Lettre US portrait");
        assert_eq!(nom_format(100.0, 150.5), "100 × 150,5 mm");
    }

    #[test]
    fn formats_are_grouped_by_runs() {
        let g = regrouper_formats(&[(210.0, 297.0), (210.0, 297.0), (297.0, 210.0), (210.0, 297.0)]);
        let pages: Vec<_> = g.iter().map(|f| f.pages.as_str()).collect();
        assert_eq!(pages, ["1-2", "3", "4"]);
    }
}
