//! Screen tests of the window (egui_kittest, rendered by wgpu): the main screen with an invented
//! batch of minutes, then the four steps of the cut at each header. The images are compared with
//! tests/snapshots/ (`UPDATE_SNAPSHOTS=1` rewrites them).

use std::path::{Path, PathBuf};

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use massicot::fenetre::Atelier;
use massicot_essais::{Ligne, Page, document, octets};

/// 12 invented pages: PV 12 (1-3), 13 (4-5), 14 (6-8, page 8 nearly empty), 15 (9-10), 16 (11-12).
/// Every page has a text layer: no OCR, so the same report wherever Tesseract is installed or not.
fn lot(dossier: &Path) -> PathBuf {
    let textes = [
        "L'assemblée s'est réunie à la mairie, sous la présidence du maire.",
        "Étaient présents : neuf conseillers. Excusés : deux.",
        "Le compte rendu de la séance précédente est approuvé à l'unanimité.",
        "Le conseil approuve le budget annexe et la convention de voirie.",
        "Questions diverses : éclairage public, cantine, travaux de l'école.",
    ];
    let debuts = [(1, 12), (4, 13), (6, 14), (9, 15), (11, 16)];
    let pages: Vec<Page> = (1..=12)
        .map(|n| {
            if n == 8 {
                return Page::a4(vec![Ligne::new(60.0, 770.0, 11.0, "Fin de la séance à 22 h.")]);
            }
            let mut lignes = Vec::new();
            let mut y = 770.0;
            if let Some((_, pv)) = debuts.iter().find(|(p, _)| *p == n) {
                lignes.push(Ligne::new(60.0, y, 18.0, format!("PROCÈS-VERBAL N° {pv}")));
                y -= 50.0;
            }
            for k in 0..24 {
                lignes.push(Ligne::new(60.0, y - 24.0 * k as f64, 11.0, textes[(n + k) % textes.len()]));
            }
            Page::a4(lignes)
        })
        .collect();
    std::fs::create_dir_all(dossier).unwrap();
    let chemin = dossier.join("lot-pv.pdf");
    std::fs::write(&chemin, octets(&document(&pages, Some("Lot inventé"), &[]))).unwrap();
    chemin
}

/// A fixed folder: the paths shown in the window are the same at every run.
fn dossier(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(nom);
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn harnais(pdf: &Path) -> Harness<'static, Atelier> {
    let pdf = pdf.to_path_buf();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1180.0, 760.0))
        .with_render_cursor(false)
        .build_eframe(move |cc| {
            let mut a = Atelier::new(&cc.egui_ctx, Some(&pdf));
            a.sans_durees();
            a
        });
    attendre(&mut h, Atelier::vignettes_pretes);
    h
}

/// Runs frames until the window is ready (thumbnails drawn, tool finished).
fn attendre(h: &mut Harness<'static, Atelier>, pret: fn(&Atelier) -> bool) {
    for _ in 0..3000 {
        h.step();
        if pret(h.state()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(pret(h.state()), "la fenêtre n'est pas prête à temps");
    h.run_steps(4);
}

#[test]
fn main_screen_with_an_open_pdf() {
    let pdf = lot(&dossier("massicot-fenetre-principale"));
    let mut h = harnais(&pdf);
    h.snapshot("fenetre-principale");
}

#[test]
fn cut_at_each_header_in_four_steps() {
    let pdf = lot(&dossier("massicot-fenetre-decoupe"));
    let mut h = harnais(&pdf);
    h.get_by_label("Découpe par en-tête").click();
    h.run_steps(4);
    h.snapshot("fenetre-decoupe-1-pdf");

    h.get_by_label("Suivant ›").click();
    h.run_steps(4);
    h.snapshot("fenetre-decoupe-2-en-tete");

    h.get_by_label("Voir le rapport (rien n'est écrit)").click();
    h.run_steps(2);
    attendre(&mut h, Atelier::libre);
    h.snapshot("fenetre-decoupe-3-rapport");

    h.get_by_label("Couper en 5 fichiers").click();
    h.run_steps(2);
    attendre(&mut h, Atelier::libre);
    h.snapshot("fenetre-decoupe-4-couper");
    let ecrits = std::fs::read_dir(pdf.with_file_name("lot-pv-pv")).unwrap().count();
    assert_eq!(ecrits, 6, "5 PV et le rapport");
}

#[test]
fn extract_the_selected_pages() {
    let pdf = lot(&dossier("massicot-fenetre-extraire"));
    let mut h = harnais(&pdf);
    h.get_by_label("Extraire des pages").click();
    h.run_steps(2);
    h.state_mut().selectionner_pour_essai(&[1, 2, 4]);
    h.run_steps(4);
    h.snapshot("fenetre-extraire");
    h.get_by_label("Extraire").click();
    h.run_steps(2);
    attendre(&mut h, Atelier::libre);
    h.snapshot("fenetre-extraire-fait");
    let sortie = pdf.with_file_name("lot-pv-extrait.pdf");
    let infos = std::process::Command::new(env!("CARGO_BIN_EXE_massicot")).args(["infos", "--json"]).arg(&sortie).output().unwrap();
    assert!(String::from_utf8_lossy(&infos.stdout).contains("\"pages\": 3"), "{}", String::from_utf8_lossy(&infos.stdout));
}
