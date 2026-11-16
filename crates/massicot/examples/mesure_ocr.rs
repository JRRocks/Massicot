//! Measures what the OCR engines read from synthetic scanned PV headers with accents.
//! `cargo run --release -p massicot --example mesure_ocr [integre|tesseract]`

use std::path::Path;
use std::sync::Arc;

use massicot::lecture::{Lecteur, MoteurOcr};
use massicot_essais::{Ligne, Page, Scan, document, octets, scanner};

fn main() {
    let moteur = match std::env::args().nth(1).as_deref() {
        Some("tesseract") => MoteurOcr::Tesseract,
        _ => MoteurOcr::Integre,
    };
    let entetes = [
        "PROCÈS-VERBAL N° 12",
        "PROCÈS-VERBAL N° 3",
        "Procès-verbal de la séance du 14 mars 2025",
        "PV N° 47 - ÉLECTION DU BUREAU",
        "Délibération n° 2025-018",
    ];
    for (k, reglage) in [Scan { dpi: 200.0, bruit: 20, inclinaison: 0.0, graine: 3 }, Scan { dpi: 150.0, bruit: 60, inclinaison: 0.8, graine: 5 }]
        .into_iter()
        .enumerate()
    {
        let pages: Vec<Page> = entetes
            .iter()
            .map(|e| {
                Page::a4(vec![
                    Ligne::new(60.0, 770.0, 18.0, *e),
                    Ligne::new(60.0, 720.0, 11.0, "L'assemblée générale s'est réunie à 18 h 30, présidée par un élu."),
                    Ligne::new(60.0, 700.0, 11.0, "Étaient présents : douze membres ; excusés : deux."),
                ])
            })
            .collect();
        let scan = octets(&scanner(&octets(&document(&pages, None, &[])), reglage));
        let mut l = Lecteur::depuis_octets(Path::new("mesure.pdf"), Arc::new(scan), moteur).unwrap();
        println!("== réglage {k} : {} dpi, bruit {}, inclinaison {}°", reglage.dpi, reglage.bruit, reglage.inclinaison);
        for (i, e) in entetes.iter().enumerate() {
            let p = l.ocr(i).unwrap();
            let lu: Vec<String> = p.mots.iter().filter(|m| m.rect[1] < 120.0).map(|m| format!("{}@{:.0},{:.0}", m.texte, m.rect[0], m.rect[1])).collect();
            println!("attendu : {e:<45} haut de page lu : {}", lu.join(" "));
            if std::env::var_os("TOUT").is_some() { println!("{}", p.texte); }
        }
    }
}
