//! `massicot-essais <dossier>`: writes invented test PDFs into a folder (used by the CI):
//! `essai.pdf` (5 pages of text) and `essai-scan.pdf` (the same pages as pictures only).

use std::path::PathBuf;

use massicot_essais::{Ligne, Page, Scan, document, octets, scanner};

fn main() {
    let dossier = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "essais".into()));
    std::fs::create_dir_all(&dossier).expect("dossier");
    let pages: Vec<Page> = (1..=5)
        .map(|i| Page::a4(vec![Ligne::new(72.0, 770.0, 18.0, format!("PROCÈS-VERBAL N° {i}")), Ligne::new(72.0, 700.0, 11.0, "Texte inventé.")]))
        .collect();
    let doc = document(&pages, Some("Essai Massicot"), &[("Premier", 0), ("Troisième", 2)]);
    let texte = octets(&doc);
    std::fs::write(dossier.join("essai.pdf"), &texte).expect("écriture");
    std::fs::write(dossier.join("essai-scan.pdf"), octets(&scanner(&texte, Scan::default()))).expect("écriture");
    println!("{}", dossier.display());
}
