//! `massicot infos` on generated PDFs.

use std::process::Command;

use massicot_essais::{Ligne, Page, document, octets};

#[test]
fn infos_describes_pages_formats_metadata_and_bookmarks() {
    let dir = tempfile::tempdir().unwrap();
    let mut pages: Vec<Page> = (1..=3).map(|i| Page::a4(vec![Ligne::new(72.0, 770.0, 18.0, format!("Page {i}"))])).collect();
    pages.push(Page { largeur: 842.0, hauteur: 595.0, lignes: vec![] });
    let doc = document(&pages, Some("Procès-verbaux d'essai"), &[("Début", 0), ("Fin", 3)]);
    let chemin = dir.path().join("essai.pdf");
    std::fs::write(&chemin, octets(&doc)).unwrap();

    let infos = massicot::ops::infos::infos(&chemin).unwrap();
    assert_eq!(infos.pages, 4);
    assert!(!infos.protege);
    assert_eq!(infos.metadonnees.get("titre").map(String::as_str), Some("Procès-verbaux d'essai"));
    assert_eq!(infos.signets, 2);
    let formats: Vec<_> = infos.formats.iter().map(|f| (f.pages.as_str(), f.format.as_str())).collect();
    assert_eq!(formats, [("1-3", "A4 portrait"), ("4", "A4 paysage")]);

    let out = Command::new(env!("CARGO_BIN_EXE_massicot")).args(["infos", "--json"]).arg(&chemin).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["pages"], 4);
    assert_eq!(json["formats"][1]["format"], "A4 paysage");

    let out = Command::new(env!("CARGO_BIN_EXE_massicot")).arg("infos").arg(&chemin).output().unwrap();
    let texte = String::from_utf8(out.stdout).unwrap();
    assert!(texte.contains("Pages : 4") && texte.contains("Titre : Procès-verbaux d'essai"), "{texte}");
}

#[test]
fn infos_reports_missing_and_non_pdf_files_in_french() {
    let dir = tempfile::tempdir().unwrap();
    let absent = dir.path().join("absent.pdf");
    let e = massicot::ops::infos::infos(&absent).unwrap_err();
    assert!(e.message().contains("introuvable"), "{e}");

    let texte = dir.path().join("note.pdf");
    std::fs::write(&texte, "ceci n'est pas un PDF").unwrap();
    let e = massicot::ops::infos::infos(&texte).unwrap_err();
    assert!(e.message().contains("n'est pas un fichier PDF"), "{e}");

    let out = Command::new(env!("CARGO_BIN_EXE_massicot")).arg("infos").arg(&texte).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("massicot : "));
}
