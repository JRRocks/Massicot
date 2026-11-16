//! `decouper-regex` on invented batches of minutes (PV): text PDFs and "scanned" PDFs (pictures
//! only, read by OCR), with noise, accents, a cover page, a blank page, a repeated number and a
//! header-like line at the bottom of a page.

use std::path::{Path, PathBuf};
use std::process::Command;

use massicot::lecture::{ModeOcr, MoteurOcr};
use massicot::ops::entetes::{AvantPremier, Rapport, Reglages, decouper_regex};
use massicot_essais::{Ligne, Page, Scan, document, octets, scanner};

const REGEX_PV: &str = r"PROCÈS-VERBAL N° (?<num>\d+)";
const REGEX_DELIB: &str = r"DÉLIBÉRATION N° (?<num>\d+)";

fn corps(lignes: &mut Vec<Ligne>, depuis: f64) {
    let textes = [
        "L'assemblée s'est réunie à la mairie, sous la présidence du maire.",
        "Étaient présents : neuf conseillers. Excusés : deux.",
        "Le compte rendu de la séance précédente est approuvé à l'unanimité.",
    ];
    for (k, t) in textes.iter().enumerate() {
        lignes.push(Ligne::new(60.0, depuis - 18.0 * k as f64, 11.0, *t));
    }
}

/// The batch: 8 pages.
/// 1 cover · 2-3 PV 12 (page 3 mentions « voir le PV n° 11 ») · 4 « Procès-verbal n° 13 » ·
/// 5 blank · 6 DÉLIBÉRATION N° 7 · 7 PV 12 again · 8 continued, with « PROCÈS-VERBAL N° 99 »
/// quoted near the bottom of the page.
fn lot() -> Vec<Page> {
    let mut pages = Vec::new();
    pages.push(Page::a4(vec![Ligne::new(60.0, 700.0, 22.0, "Registre des délibérations 2025")]));
    let mut p = vec![Ligne::new(60.0, 770.0, 18.0, "PROCÈS-VERBAL N° 12")];
    corps(&mut p, 720.0);
    pages.push(Page::a4(p));
    let mut p = vec![];
    corps(&mut p, 760.0);
    p.push(Ligne::new(60.0, 400.0, 11.0, "Voir aussi le PV n° 11 pour les questions diverses."));
    pages.push(Page::a4(p));
    let mut p = vec![Ligne::new(60.0, 770.0, 18.0, "Procès-verbal n° 13")];
    corps(&mut p, 720.0);
    pages.push(Page::a4(p));
    pages.push(Page::a4(vec![]));
    let mut p = vec![Ligne::new(60.0, 770.0, 18.0, "DÉLIBÉRATION N° 7")];
    corps(&mut p, 720.0);
    pages.push(Page::a4(p));
    let mut p = vec![Ligne::new(60.0, 770.0, 18.0, "PROCÈS-VERBAL N° 12")];
    corps(&mut p, 720.0);
    pages.push(Page::a4(p));
    let mut p = vec![];
    corps(&mut p, 760.0);
    p.push(Ligne::new(60.0, 120.0, 11.0, "Copie conforme au PROCÈS-VERBAL N° 99 du registre."));
    pages.push(Page::a4(p));
    pages
}

fn ecrire(dir: &Path, nom: &str, contenu: Vec<u8>) -> PathBuf {
    let chemin = dir.join(nom);
    std::fs::write(&chemin, contenu).unwrap();
    chemin
}

fn lot_texte(dir: &Path) -> PathBuf {
    ecrire(dir, "lot.pdf", octets(&document(&lot(), None, &[])))
}

fn lot_scanne(dir: &Path) -> PathBuf {
    let texte = octets(&document(&lot(), None, &[]));
    ecrire(dir, "lot-scanne.pdf", octets(&scanner(&texte, Scan { dpi: 200.0, bruit: 10, inclinaison: 0.0, graine: 9 })))
}

fn reglages(zone: Option<u8>) -> Reglages {
    Reglages {
        modele: "PV-{num}.pdf".into(),
        zone_haute: zone,
        ..Reglages::new(vec![REGEX_PV.into(), REGEX_DELIB.into()])
    }
}

/// (file, pages) of each block.
fn blocs(r: &Rapport) -> Vec<(String, String)> {
    r.blocs.iter().map(|b| (b.fichier.clone(), b.pages.clone())).collect()
}

fn attendu(avec_99: bool) -> Vec<(String, String)> {
    let mut v = vec![
        ("00-pages-avant-le-premier-en-tete.pdf", "1"),
        ("PV-12.pdf", "2-3"),
        ("PV-13.pdf", "4-5"),
        ("PV-7.pdf", "6"),
    ];
    if avec_99 {
        v.extend([("PV-12-2.pdf", "7"), ("PV-99.pdf", "8")]);
    } else {
        v.push(("PV-12-2.pdf", "7-8"));
    }
    v.into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

#[test]
fn dry_run_on_a_text_batch_reports_each_block() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_texte(d.path());
    let r = decouper_regex(&lot, &d.path().join("sortie"), &reglages(Some(30)), None, false).unwrap();
    assert!(r.a_blanc);
    assert_eq!(blocs(&r), attendu(false));
    assert!(!d.path().join("sortie").exists(), "the dry run writes nothing");

    let pv12 = r.blocs[1].en_tete.as_ref().unwrap();
    assert_eq!(pv12.texte_trouve, "PROCÈS-VERBAL N° 12");
    assert_eq!(pv12.groupes.get("num").map(String::as_str), Some("12"));
    assert_eq!(pv12.regex, 1);
    assert_eq!(r.blocs[3].en_tete.as_ref().unwrap().regex, 2);
    assert_eq!(r.blocs[2].en_tete.as_ref().unwrap().texte_trouve, "Procès-verbal n° 13");
    assert!(r.blocs[1].extrait.starts_with("PROCÈS-VERBAL N° 12 L'assemblée"), "{}", r.blocs[1].extrait);
    // Alerts: the blank page, the repeated name, the pages before the first header.
    assert!(r.blocs[2].alertes.iter().any(|a| a.contains("page 5 : aucun texte")), "{:?}", r.blocs[2].alertes);
    assert!(r.blocs[4].alertes.iter().any(|a| a.contains("« PV-12.pdf » déjà pris")), "{:?}", r.blocs[4].alertes);
    assert!(r.alertes.iter().any(|a| a.contains("avant le premier en-tête")), "{:?}", r.alertes);
    // Only the blank page has no text layer: in « auto » mode, it alone goes through OCR.
    assert_eq!(r.pages_lues_par_ocr, 1);
}

#[test]
fn without_a_top_band_a_quoted_header_at_the_bottom_cuts() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_texte(d.path());
    let r = decouper_regex(&lot, &d.path().join("s"), &reglages(None), None, false).unwrap();
    assert_eq!(blocs(&r), attendu(true));
}

#[test]
fn pages_before_the_first_header_can_join_it_or_be_left_out() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_texte(d.path());
    let r = decouper_regex(&lot, &d.path().join("s"), &Reglages { avant_premier: AvantPremier::Premier, ..reglages(Some(30)) }, None, false).unwrap();
    assert_eq!((r.blocs[0].fichier.as_str(), r.blocs[0].pages.as_str()), ("PV-12.pdf", "1-3"));
    let r = decouper_regex(&lot, &d.path().join("s"), &Reglages { avant_premier: AvantPremier::Ignorer, ..reglages(Some(30)) }, None, false).unwrap();
    assert_eq!(r.blocs[0].pages, "2-3");
    assert_eq!(r.pages_ignorees, [1]);
}

#[test]
fn case_and_accents_count_only_when_asked() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_texte(d.path());
    let strict = Reglages { respecter_casse: true, respecter_accents: true, ..reglages(Some(30)) };
    let r = decouper_regex(&lot, &d.path().join("s"), &strict, None, false).unwrap();
    // « Procès-verbal n° 13 » no longer matches: page 4 stays in block PV-12.
    assert!(r.blocs.iter().all(|b| b.fichier != "PV-13.pdf"));
    let r = decouper_regex(&lot, &d.path().join("s"), &Reglages { regex: vec![r"proces-verbal n° (?<num>\d+)".into()], ..reglages(Some(30)) }, None, false).unwrap();
    assert!(r.blocs.iter().any(|b| b.fichier == "PV-13.pdf"));
}

#[test]
fn the_real_cut_needs_the_dry_run_token_and_writes_files_and_report() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_texte(d.path());
    let sortie = d.path().join("PV");
    let r = reglages(Some(30));
    let blanc = decouper_regex(&lot, &sortie, &r, None, false).unwrap();

    let e = decouper_regex(&lot, &sortie, &r, Some("000000000000"), false).unwrap_err();
    assert!(e.message().contains("jeton inconnu"), "{e}");
    let autre = Reglages { zone_haute: Some(40), ..reglages(Some(30)) };
    assert!(decouper_regex(&lot, &sortie, &autre, Some(&blanc.jeton), false).is_err(), "other settings, other token");
    assert!(!sortie.exists());

    let fait = decouper_regex(&lot, &sortie, &r, Some(&blanc.jeton), false).unwrap();
    assert!(!fait.a_blanc);
    assert_eq!(blocs(&fait), attendu(false));
    for (fichier, pages) in attendu(false) {
        let doc = massicot::pdf::ouvrir(&sortie.join(&fichier)).unwrap();
        let attendues = massicot::pdf::pages(&pages, 8).unwrap().len();
        assert_eq!(massicot::pdf::nombre_de_pages(&doc).unwrap(), attendues, "{fichier}");
    }
    let rapport: serde_json::Value = serde_json::from_slice(&std::fs::read(sortie.join("rapport-decoupe.json")).unwrap()).unwrap();
    assert_eq!(rapport["blocs"][1]["fichier"], "PV-12.pdf");
    assert_eq!(rapport["blocs"][1]["en_tete"]["groupes"]["num"], "12");
    assert_eq!(rapport["a_blanc"], false);

    // A second cut into the same folder refuses to replace the files.
    let e = decouper_regex(&lot, &sortie, &r, Some(&blanc.jeton), false).unwrap_err();
    assert!(e.message().contains("--ecraser"), "{e}");
}

#[test]
fn a_scanned_batch_is_read_by_ocr_and_cut_the_same_way() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_scanne(d.path());
    let r = Reglages { moteur: MoteurOcr::Integre, ..reglages(Some(30)) };
    let rapport = decouper_regex(&lot, &d.path().join("s"), &r, None, false).unwrap();
    assert_eq!(rapport.pages_lues_par_ocr, 8);
    assert_eq!(rapport.moteur_ocr.as_deref(), Some("integre"));
    assert_eq!(blocs(&rapport), attendu(false), "{}", rapport.texte());
    // The built-in engine reads « È » and « ° » as « ? »: the header is found all the same, and flagged.
    let pv12 = rapport.blocs[1].en_tete.as_ref().unwrap();
    assert!(pv12.texte_trouve.contains('?') && pv12.texte_trouve.ends_with("12"), "{pv12:?}");
    assert!(rapport.blocs[1].alertes.iter().any(|a| a.contains("moteur OCR intégré")));

    // OCR never: a scan has no text layer, so no header.
    let jamais = decouper_regex(&lot, &d.path().join("s"), &Reglages { ocr: ModeOcr::Jamais, ..r }, None, false).unwrap();
    assert!(jamais.blocs.is_empty());
    assert!(jamais.alertes.iter().any(|a| a.contains("aucun en-tête trouvé")));
}

/// Tesseract reads the accents; it runs where it is installed (always in the CI).
#[test]
fn a_scanned_batch_read_by_tesseract_keeps_the_accents() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_scanne(d.path());
    let r = Reglages { moteur: MoteurOcr::Tesseract, ..reglages(Some(30)) };
    let rapport = match decouper_regex(&lot, &d.path().join("s"), &r, None, false) {
        Ok(rapport) => rapport,
        Err(e) => {
            assert!(std::env::var_os("MASSICOT_EXIGER_TESSERACT").is_none(), "Tesseract exigé : {e}");
            eprintln!("Tesseract absent : essai sauté");
            return;
        }
    };
    assert_eq!(blocs(&rapport), attendu(false), "{}", rapport.texte());
    assert_eq!(rapport.blocs[1].en_tete.as_ref().unwrap().texte_trouve, "PROCÈS-VERBAL N° 12");
    assert!(rapport.blocs[1].en_tete.as_ref().unwrap().confiance.is_some());
}

#[test]
fn the_command_line_dry_run_then_cut() {
    let d = tempfile::tempdir().unwrap();
    let lot = lot_texte(d.path());
    let sortie = d.path().join("PV");
    let exe = env!("CARGO_BIN_EXE_massicot");
    let base = |cmd: &mut Command| {
        cmd.arg("decouper-regex").arg(&lot).args(["--regex", REGEX_PV, "--regex", REGEX_DELIB, "--zone-haute", "30", "--modele", "PV-{num}.pdf", "--dossier"]).arg(&sortie);
    };
    let mut c = Command::new(exe);
    base(&mut c);
    let out = c.output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let texte = String::from_utf8(out.stdout).unwrap();
    assert!(texte.contains("Mode à blanc") && texte.contains("PV-13.pdf : pages 4-5"), "{texte}");
    let jeton = texte.lines().find_map(|l| l.strip_prefix("Pour couper : mêmes options, plus --couper ")).unwrap().trim().to_string();

    let mut c = Command::new(exe);
    base(&mut c);
    let out = c.args(["--couper", &jeton, "--json"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["blocs"].as_array().unwrap().len(), 5);
    assert!(sortie.join("PV-7.pdf").is_file() && sortie.join("rapport-decoupe.json").is_file());

    let out = Command::new(exe).arg("decouper-regex").arg(&lot).args(["--regex", "PV(", "--dossier", "x"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("expression régulière « PV( » invalide"));
}
