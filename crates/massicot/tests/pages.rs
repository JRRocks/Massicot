//! Merge, extract, split, rotate, delete, move and reverse, on generated PDFs whose page `i`
//! (1-based) is `500 + i` points wide, so the order of the pages can be read back.

use std::path::{Path, PathBuf};
use std::process::Command;

use massicot::ops::decoupe::{Mode, decouper};
use massicot::ops::fusion::fusionner;
use massicot::ops::pages::{Position, Selection, deplacer, extraire, inverser, supprimer, tourner};
use massicot_essais::{Ligne, Page, document, octets};

fn page(i: usize, lignes: Vec<Ligne>) -> Page {
    Page { largeur: 500.0 + i as f64, hauteur: 800.0, lignes }
}

/// A PDF of `n` pages, page i being 500 + i points wide, with these top-level bookmarks.
fn pdf(dir: &Path, nom: &str, n: usize, signets: &[(&str, usize)]) -> PathBuf {
    let pages: Vec<Page> = (1..=n).map(|i| page(i, vec![Ligne::new(50.0, 700.0, 20.0, format!("Page {i}"))])).collect();
    let chemin = dir.join(nom);
    std::fs::write(&chemin, octets(&document(&pages, Some(nom), signets))).unwrap();
    chemin
}

fn ouvrir(chemin: &Path) -> pdfcraft_cos::Document {
    massicot::pdf::ouvrir(chemin).unwrap()
}

/// Page widths minus 500: the original page numbers.
fn ordre(chemin: &Path) -> Vec<usize> {
    pdfcraft_organize::page_boxes(&ouvrir(chemin)).unwrap().iter().map(|b| (b[0][2] - b[0][0]).round() as usize - 500).collect()
}

fn rotations(chemin: &Path) -> Vec<i64> {
    let doc = ouvrir(chemin);
    (0..massicot::pdf::nombre_de_pages(&doc).unwrap()).map(|i| pdfcraft_organize::page_rotation(&doc, i).unwrap()).collect()
}

fn titres_signets(chemin: &Path) -> Vec<(usize, String)> {
    pdfcraft_organize::bookmarks(&ouvrir(chemin)).into_iter().map(|b| (b.path.len(), b.title)).collect()
}

#[test]
fn extraction_keeps_the_given_order() {
    let d = tempfile::tempdir().unwrap();
    let src = pdf(d.path(), "src.pdf", 6, &[]);
    let r = extraire(&src, "5,1-2,fin", &d.path().join("x.pdf"), false).unwrap();
    assert_eq!(r.pages, 4);
    assert_eq!(ordre(&d.path().join("x.pdf")), [5, 1, 2, 6]);
}

#[test]
fn merge_keeps_order_and_adds_one_bookmark_per_file() {
    let d = tempfile::tempdir().unwrap();
    let a = pdf(d.path(), "a.pdf", 3, &[("A deux", 1)]);
    let b = pdf(d.path(), "b.pdf", 2, &[]);
    let out = d.path().join("ab.pdf");
    let r = fusionner(&[a.clone(), b.clone()], &out, false, false).unwrap();
    assert_eq!(r.pages, 5);
    assert_eq!(ordre(&out), [1, 2, 3, 1, 2]);
    assert_eq!(titres_signets(&out), [(1, "a".to_string()), (2, "A deux".to_string()), (1, "b".to_string())]);

    let nu = d.path().join("nu.pdf");
    fusionner(&[a, b], &nu, true, false).unwrap();
    assert!(titres_signets(&nu).is_empty());
}

#[test]
fn split_every_n_pages_and_before_pages() {
    let d = tempfile::tempdir().unwrap();
    let src = pdf(d.path(), "lot.pdf", 5, &[]);
    let r = decouper(&src, &Mode::ToutesLes(2), &d.path().join("par2"), None, false).unwrap();
    let noms: Vec<_> = r.fichiers.iter().map(|p| Path::new(&p.fichier).file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(noms, ["lot-01.pdf", "lot-02.pdf", "lot-03.pdf"]);
    assert_eq!(ordre(Path::new(&r.fichiers[2].fichier)), [5]);
    assert_eq!(r.fichiers.iter().map(|p| p.pages.as_str()).collect::<Vec<_>>(), ["1-2", "3-4", "5"]);

    let r = decouper(&src, &Mode::Avant("3,5".into()), &d.path().join("avant"), Some("partie"), false).unwrap();
    let parts: Vec<_> = r.fichiers.iter().map(|p| ordre(Path::new(&p.fichier))).collect();
    assert_eq!(parts, [vec![1, 2], vec![3, 4], vec![5]]);
    assert!(r.fichiers[0].fichier.ends_with("partie-01.pdf"));
}

#[test]
fn split_at_bookmarks_of_a_chosen_level() {
    let d = tempfile::tempdir().unwrap();
    let pages: Vec<Page> = (1..=6).map(|i| page(i, vec![])).collect();
    let mut doc = document(&pages, None, &[("Chapitre : un", 1), ("Chapitre deux", 3)]);
    pdfcraft_organize::add_bookmark(&mut doc, &[0], 0, "Section 1.1", 2).unwrap();
    pdfcraft_organize::add_bookmark(&mut doc, &[1], 0, "Section 2.1", 4).unwrap();
    let src = d.path().join("livre.pdf");
    std::fs::write(&src, octets(&doc)).unwrap();

    let r = decouper(&src, &Mode::Signets(1), &d.path().join("n1"), None, false).unwrap();
    let noms: Vec<_> = r.fichiers.iter().map(|p| Path::new(&p.fichier).file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(noms, ["01-début.pdf", "02-Chapitre _ un.pdf", "03-Chapitre deux.pdf"]);
    let parts: Vec<_> = r.fichiers.iter().map(|p| ordre(Path::new(&p.fichier))).collect();
    assert_eq!(parts, [vec![1], vec![2, 3], vec![4, 5, 6]]);

    let r = decouper(&src, &Mode::Signets(2), &d.path().join("n2"), None, false).unwrap();
    let parts: Vec<_> = r.fichiers.iter().map(|p| ordre(Path::new(&p.fichier))).collect();
    assert_eq!(parts, [vec![1, 2], vec![3, 4], vec![5, 6]]);
    assert_eq!(r.fichiers[2].signet.as_deref(), Some("Section 2.1"));

    let e = decouper(&src, &Mode::Signets(3), &d.path().join("n3"), None, false).unwrap_err();
    assert!(e.message().contains("aucun signet de niveau 3"), "{e}");
}

#[test]
fn split_by_size_keeps_every_part_under_the_limit() {
    let d = tempfile::tempdir().unwrap();
    // Pseudo-random letters compress badly: each page weighs a few kilobytes.
    let mut graine: u64 = 7;
    let mut lettres = |n: usize| -> String {
        (0..n)
            .map(|_| {
                graine = graine.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (b'a' + (graine >> 59) as u8 % 26) as char
            })
            .collect()
    };
    let pages: Vec<Page> =
        (1..=8).map(|i| page(i, (0..30).map(|l| Ligne::new(20.0, 780.0 - 20.0 * l as f64, 8.0, lettres(100))).collect())).collect();
    let src = d.path().join("gros.pdf");
    std::fs::write(&src, octets(&document(&pages, None, &[]))).unwrap();
    let limite = 6 * 1024;
    let r = decouper(&src, &Mode::Taille(limite), &d.path().join("t"), None, false).unwrap();
    assert!(r.fichiers.len() > 1, "{r:?}");
    let mut toutes = Vec::new();
    for p in &r.fichiers {
        assert!(p.taille_octets <= limite || p.nombre_pages == 1, "{p:?}");
        toutes.extend(ordre(Path::new(&p.fichier)));
    }
    assert_eq!(toutes, (1..=8).collect::<Vec<_>>());
}

#[test]
fn rotate_even_odd_or_selected_pages() {
    let d = tempfile::tempdir().unwrap();
    let src = pdf(d.path(), "r.pdf", 4, &[]);
    tourner(&src, 90, &Selection::Paires, &d.path().join("p.pdf"), false).unwrap();
    assert_eq!(rotations(&d.path().join("p.pdf")), [0, 90, 0, 90]);
    tourner(&src, -90, &Selection::Impaires, &d.path().join("i.pdf"), false).unwrap();
    assert_eq!(rotations(&d.path().join("i.pdf")), [270, 0, 270, 0]);
    tourner(&src, 180, &Selection::Pages("2-3".into()), &d.path().join("s.pdf"), false).unwrap();
    assert_eq!(rotations(&d.path().join("s.pdf")), [0, 180, 180, 0]);
    tourner(&src, 270, &Selection::Toutes, &d.path().join("t.pdf"), false).unwrap();
    assert_eq!(rotations(&d.path().join("t.pdf")), [270; 4]);
}

#[test]
fn delete_move_and_reverse_pages() {
    let d = tempfile::tempdir().unwrap();
    let src = pdf(d.path(), "m.pdf", 6, &[]);
    supprimer(&src, &Selection::Impaires, &d.path().join("s.pdf"), false).unwrap();
    assert_eq!(ordre(&d.path().join("s.pdf")), [2, 4, 6]);
    let e = supprimer(&src, &Selection::Toutes, &d.path().join("s2.pdf"), false).unwrap_err();
    assert!(e.message().contains("toutes les pages"), "{e}");

    deplacer(&src, "5-6", Position::Debut, &d.path().join("d1.pdf"), false).unwrap();
    assert_eq!(ordre(&d.path().join("d1.pdf")), [5, 6, 1, 2, 3, 4]);
    deplacer(&src, "1", Position::A(3), &d.path().join("d2.pdf"), false).unwrap();
    assert_eq!(ordre(&d.path().join("d2.pdf")), [2, 3, 1, 4, 5, 6]);
    deplacer(&src, "2", Position::Fin, &d.path().join("d3.pdf"), false).unwrap();
    assert_eq!(ordre(&d.path().join("d3.pdf")), [1, 3, 4, 5, 6, 2]);
    let e = deplacer(&src, "2", Position::A(7), &d.path().join("d4.pdf"), false).unwrap_err();
    assert!(e.message().contains("entre 1 et 6"), "{e}");

    inverser(&src, &Selection::Toutes, &d.path().join("i1.pdf"), false).unwrap();
    assert_eq!(ordre(&d.path().join("i1.pdf")), [6, 5, 4, 3, 2, 1]);
    inverser(&src, &Selection::Pages("2-4".into()), &d.path().join("i2.pdf"), false).unwrap();
    assert_eq!(ordre(&d.path().join("i2.pdf")), [1, 4, 3, 2, 5, 6]);
}

#[test]
fn existing_outputs_are_never_replaced_without_permission() {
    let d = tempfile::tempdir().unwrap();
    let src = pdf(d.path(), "e.pdf", 2, &[]);
    let out = d.path().join("deja.pdf");
    std::fs::write(&out, b"ne pas perdre").unwrap();
    let e = extraire(&src, "1", &out, false).unwrap_err();
    assert!(e.message().contains("--ecraser"), "{e}");
    assert_eq!(std::fs::read(&out).unwrap(), b"ne pas perdre");
    extraire(&src, "1", &out, true).unwrap();
    assert_eq!(ordre(&out), [1]);

    // A split checks every name before writing the first file.
    std::fs::create_dir(d.path().join("lot")).unwrap();
    std::fs::write(d.path().join("lot/e-02.pdf"), b"x").unwrap();
    assert!(decouper(&src, &Mode::ToutesLes(1), &d.path().join("lot"), None, false).is_err());
    assert!(!d.path().join("lot/e-01.pdf").exists());
}

#[test]
fn the_command_line_runs_a_split_and_explains_bad_arguments_in_french() {
    let d = tempfile::tempdir().unwrap();
    let src = pdf(d.path(), "cli.pdf", 4, &[]);
    let exe = env!("CARGO_BIN_EXE_massicot");
    let out = Command::new(exe).args(["decouper", "--toutes", "2", "--json", "--dossier"]).arg(d.path().join("o")).arg(&src).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["fichiers"].as_array().unwrap().len(), 2);

    let out = Command::new(exe).args(["tourner", "--angle", "45", "--sortie", "x.pdf"]).arg(&src).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("l'angle doit être 90, 180, 270 ou -90"), "{err}");

    let out = Command::new(exe).args(["decouper", "--dossier", "x"]).arg(&src).output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("argument obligatoire manquant"), "{err}");

    let out = Command::new(exe).args(["decouper", "--aide"]).output().unwrap();
    let aide = String::from_utf8_lossy(&out.stdout);
    assert!(aide.contains("Un fichier toutes les N pages") && aide.contains("Utilisation :"), "{aide}");
}
