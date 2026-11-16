//! Windows file dialogs (elsewhere, the window is only built for its tests: no dialog), and
//! opening a folder in the Explorer.

use std::path::Path;

use crate::outils::Genre;

#[cfg(windows)]
fn pres_de(actuel: &str) -> rfd::FileDialog {
    let mut d = rfd::FileDialog::new();
    let p = Path::new(actuel);
    if let Some(dossier) = if p.is_dir() { Some(p) } else { p.parent() }
        && dossier.is_dir()
    {
        d = d.set_directory(dossier);
    }
    d
}

/// One PDF to open.
#[cfg(windows)]
pub fn un_pdf() -> Option<String> {
    rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_title("Ouvrir un PDF").pick_file().map(|p| p.to_string_lossy().into_owned())
}

/// Several PDFs to add.
#[cfg(windows)]
pub fn plusieurs_pdf() -> Vec<String> {
    rfd::FileDialog::new()
        .add_filter("PDF", &["pdf"])
        .set_title("Ajouter des PDF")
        .pick_files()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

/// A file or folder for a field of this kind, starting near its current value.
#[cfg(windows)]
pub fn choisir(genre: &Genre, actuel: &str) -> Option<String> {
    let d = pres_de(actuel);
    let choix = match genre {
        Genre::Dossier => d.set_title("Choisir un dossier").pick_folder(),
        Genre::Sortie => {
            let nom = Path::new(actuel).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            d.add_filter("PDF", &["pdf"]).set_title("Enregistrer sous").set_file_name(nom).save_file()
        }
        _ => d.add_filter("PDF", &["pdf"]).set_title("Choisir un PDF").pick_file(),
    };
    choix.map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(windows))]
pub fn un_pdf() -> Option<String> {
    None
}

#[cfg(not(windows))]
pub fn plusieurs_pdf() -> Vec<String> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn choisir(_genre: &Genre, _actuel: &str) -> Option<String> {
    None
}

/// Shows a folder in the Explorer.
pub fn ouvrir_dossier(dossier: &Path) {
    let programme = if cfg!(windows) { "explorer" } else { "xdg-open" };
    let _ = std::process::Command::new(programme).arg(dossier).spawn();
}
