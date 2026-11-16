//! `melanger`: fronts and backs scanned separately, interleaved into one PDF.

use std::path::{Path, PathBuf};
use std::process::Command;

use massicot::ops::melange::{Alternance, melanger};
use massicot_essais::{Page, document, octets};

/// A PDF whose page i is `base + i` points wide.
fn pdf(dir: &Path, nom: &str, n: usize, base: f64) -> PathBuf {
    let pages: Vec<Page> = (1..=n).map(|i| Page { largeur: base + i as f64, hauteur: 800.0, lignes: vec![] }).collect();
    let chemin = dir.join(nom);
    std::fs::write(&chemin, octets(&document(&pages, Some(nom), &[]))).unwrap();
    chemin
}

fn largeurs(chemin: &Path) -> Vec<usize> {
    let doc = massicot::pdf::ouvrir(chemin).unwrap();
    pdfcraft_organize::page_boxes(&doc).unwrap().iter().map(|b| (b[0][2] - b[0][0]).round() as usize).collect()
}

#[test]
fn two_files_of_three_pages_alternate() {
    let d = tempfile::tempdir().unwrap();
    let recto = pdf(d.path(), "recto.pdf", 3, 500.0);
    let verso = pdf(d.path(), "verso.pdf", 3, 600.0);
    let out = d.path().join("m.pdf");
    let r = melanger(&recto, &verso, Alternance::default(), &out, false).unwrap();
    assert_eq!(r.pages, 6);
    assert_eq!(largeurs(&out), [501, 601, 502, 602, 503, 603]);
}

#[test]
fn backs_scanned_in_reverse_order() {
    let d = tempfile::tempdir().unwrap();
    let recto = pdf(d.path(), "recto.pdf", 3, 500.0);
    let verso = pdf(d.path(), "verso.pdf", 3, 600.0);
    let out = d.path().join("m.pdf");
    melanger(&recto, &verso, Alternance { second_inverse: true, ..Alternance::default() }, &out, false).unwrap();
    assert_eq!(largeurs(&out), [501, 603, 502, 602, 503, 601]);
}

#[test]
fn uneven_files_and_steps_from_the_command_line() {
    let d = tempfile::tempdir().unwrap();
    let a = pdf(d.path(), "a.pdf", 4, 500.0);
    let b = pdf(d.path(), "b.pdf", 1, 600.0);
    let out = d.path().join("m.pdf");
    let sortie = Command::new(env!("CARGO_BIN_EXE_massicot"))
        .args(["melanger", "--pas-recto", "2", "--sortie"])
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .output()
        .unwrap();
    assert!(sortie.status.success(), "{}", String::from_utf8_lossy(&sortie.stderr));
    assert_eq!(largeurs(&out), [501, 502, 601, 503, 504]);
}
