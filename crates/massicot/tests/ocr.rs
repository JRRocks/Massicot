//! Text of pages and OCR, on invented PDFs: text pages, and "scanned" pages (pictures only).

use std::path::{Path, PathBuf};
use std::process::Command;

use massicot::lecture::{Lecteur, ModeOcr, MoteurOcr, Source};
use massicot::ops::ocr::{ocr, texte};
use massicot_essais::{Ligne, Page, Scan, document, octets, scanner};

fn pages_pv(n: usize) -> Vec<Page> {
    (1..=n)
        .map(|i| {
            Page::a4(vec![
                Ligne::new(60.0, 770.0, 18.0, format!("PROCÈS-VERBAL N° {i}")),
                Ligne::new(60.0, 720.0, 11.0, "Séance ordinaire du conseil, présidée par un élu."),
            ])
        })
        .collect()
}

fn ecrire(dir: &Path, nom: &str, octets: Vec<u8>) -> PathBuf {
    let chemin = dir.join(nom);
    std::fs::write(&chemin, octets).unwrap();
    chemin
}

fn scan_pv(dir: &Path, n: usize) -> PathBuf {
    let texte = octets(&document(&pages_pv(n), None, &[]));
    ecrire(dir, "scan.pdf", octets(&scanner(&texte, Scan::default())))
}

#[test]
fn text_layer_pages_read_with_accents_and_word_boxes() {
    let d = tempfile::tempdir().unwrap();
    let chemin = ecrire(d.path(), "pv.pdf", octets(&document(&pages_pv(2), None, &[])));
    let t = texte(&chemin, None, ModeOcr::Auto, MoteurOcr::Integre).unwrap();
    assert_eq!(t.pages.len(), 2);
    assert_eq!(t.pages[1].source, Source::CoucheTexte);
    assert!(t.pages[1].texte.contains("PROCÈS-VERBAL N° 2"), "{}", t.pages[1].texte);
    // "PROCÈS-VERBAL" is drawn 770 pt above the bottom of an 842 pt page: about 72 pt from the top.
    let mot = t.pages[0].mots.iter().find(|m| m.texte.starts_with("PROC")).unwrap();
    assert!(mot.rect[1] > 50.0 && mot.rect[3] < 80.0 && mot.rect[0] > 55.0, "{mot:?}");
}

#[test]
fn scanned_pages_have_no_text_layer_but_read_by_ocr() {
    let d = tempfile::tempdir().unwrap();
    let chemin = scan_pv(d.path(), 1);
    let sans = texte(&chemin, None, ModeOcr::Jamais, MoteurOcr::Integre).unwrap();
    assert_eq!(sans.pages[0].source, Source::Aucune);
    let lu = texte(&chemin, None, ModeOcr::Auto, MoteurOcr::Integre).unwrap();
    assert!(matches!(lu.pages[0].source, Source::Ocr { .. }));
    // The built-in engine has no accents: "PROCÈS-VERBAL N° 1" reads "PROC?S-VERBAL N? 1".
    assert!(lu.pages[0].texte.contains("S-VERBAL"), "{}", lu.pages[0].texte);
}

#[test]
fn ocr_makes_a_scanned_pdf_searchable() {
    let d = tempfile::tempdir().unwrap();
    let chemin = scan_pv(d.path(), 2);
    let sortie = d.path().join("cherchable.pdf");
    let r = ocr(&chemin, &sortie, false, MoteurOcr::Integre, 300.0, false).unwrap();
    assert_eq!(r.pages_reconnues.len(), 2);
    assert_eq!(r.moteur.as_deref(), Some("integre"));

    let mut l = Lecteur::ouvrir(&sortie, MoteurOcr::Integre).unwrap();
    let p = l.couche_texte(1).unwrap();
    assert_eq!(p.source, Source::CoucheTexte);
    assert!(p.texte.contains("S-VERBAL") && p.texte.contains('2'), "{}", p.texte);
    let mot = p.mots.iter().find(|m| m.texte.contains("VERBAL")).unwrap();
    assert!(mot.rect[1] > 40.0 && mot.rect[3] < 95.0, "{mot:?}");

    // A second pass leaves the pages alone: they have text now.
    let r = ocr(&sortie, &d.path().join("encore.pdf"), false, MoteurOcr::Integre, 300.0, false).unwrap();
    assert!(r.pages_reconnues.is_empty());
    assert_eq!(r.pages_deja_texte, [1, 2]);
}

/// Tesseract is optional: this test runs where it is installed with French (the CI installs
/// it and sets MASSICOT_EXIGER_TESSERACT, so there it cannot be skipped).
#[test]
fn tesseract_reads_french_accents_when_installed() {
    let d = tempfile::tempdir().unwrap();
    let chemin = scan_pv(d.path(), 1);
    let sortie = d.path().join("fr.pdf");
    match ocr(&chemin, &sortie, false, MoteurOcr::Tesseract, 300.0, false) {
        Ok(r) => assert_eq!(r.moteur.as_deref(), Some("tesseract")),
        Err(e) => {
            assert!(std::env::var_os("MASSICOT_EXIGER_TESSERACT").is_none(), "Tesseract exigé : {e}");
            assert!(e.message().contains("Tesseract avec le français est introuvable"), "{e}");
            eprintln!("Tesseract absent : essai sauté");
            return;
        }
    }
    let mut l = Lecteur::ouvrir(&sortie, MoteurOcr::Integre).unwrap();
    let p = l.couche_texte(0).unwrap();
    assert!(p.texte.contains("PROCÈS-VERBAL N° 1"), "{}", p.texte);
    assert!(p.texte.contains("Séance"), "{}", p.texte);
}

#[test]
fn texte_from_the_command_line() {
    let d = tempfile::tempdir().unwrap();
    let chemin = ecrire(d.path(), "pv.pdf", octets(&document(&pages_pv(3), None, &[])));
    let out = Command::new(env!("CARGO_BIN_EXE_massicot")).args(["texte", "--pages", "2-3"]).arg(&chemin).output().unwrap();
    assert!(out.status.success());
    let t = String::from_utf8(out.stdout).unwrap();
    assert!(t.contains("--- Page 2 (couche texte) ---") && t.contains("PROCÈS-VERBAL N° 3"), "{t}");
    assert!(!t.contains("Page 1 "), "{t}");
}
