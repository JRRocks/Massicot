//! Writing results: one output file, or a set of files in a folder, never overwriting by accident.

use std::path::{Path, PathBuf};

use pdfcraft_cos::Document;
use serde::Serialize;

use crate::erreur::{Erreur, Resultat};
use crate::pdf;

/// A PDF written by a command.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Ecrit {
    pub fichier: String,
    pub pages: usize,
    pub taille_octets: u64,
}

impl Ecrit {
    pub fn ligne(&self) -> String {
        let pages = if self.pages > 1 { "pages" } else { "page" };
        format!("{} ({} {pages}, {})", self.fichier, self.pages, super::taille_lisible(self.taille_octets))
    }

    pub fn texte(&self) -> String {
        format!("Écrit : {}\n", self.ligne())
    }
}

/// Refuses to replace an existing file unless `ecraser`.
pub fn verifier_cible(chemin: &Path, ecraser: bool) -> Resultat<()> {
    if chemin.exists() && !ecraser {
        return Err(Erreur::new(format!(
            "« {} » existe déjà : choisissez un autre nom, ou ajoutez --ecraser pour le remplacer",
            chemin.display()
        )));
    }
    Ok(())
}

/// Saves `doc` to `chemin` (after [`verifier_cible`]).
pub fn ecrire_pdf(doc: &Document, chemin: &Path, ecraser: bool) -> Resultat<Ecrit> {
    verifier_cible(chemin, ecraser)?;
    let pages = pdf::nombre_de_pages(doc)?;
    let taille = pdf::enregistrer(doc, chemin)?;
    Ok(Ecrit { fichier: chemin.display().to_string(), pages, taille_octets: taille })
}

/// Saves several documents in `dossier` under the given names. Every target is checked before
/// the first file is written, so a refused run writes nothing.
pub fn ecrire_lot(docs: &[(String, Document)], dossier: &Path, ecraser: bool) -> Resultat<Vec<Ecrit>> {
    let chemins: Vec<PathBuf> = docs.iter().map(|(nom, _)| dossier.join(nom)).collect();
    for c in &chemins {
        verifier_cible(c, ecraser)?;
    }
    docs.iter().zip(&chemins).map(|((_, doc), c)| ecrire_pdf(doc, c, true)).collect()
}

/// A file name made safe for Windows: no reserved characters, no trailing dot or space,
/// at most 120 characters; empty names become `document`.
pub fn nom_sur(nom: &str) -> String {
    let mut s: String = nom
        .chars()
        .map(|c| if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { '_' } else { c })
        .collect();
    s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut s: String = s.chars().take(120).collect();
    while s.ends_with('.') || s.ends_with(' ') {
        s.pop();
    }
    let reserve = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "LPT1", "LPT2", "LPT3"];
    let base = s.split('.').next().unwrap_or("").to_ascii_uppercase();
    if s.is_empty() || reserve.contains(&base.as_str()) {
        s = if s.is_empty() { "document".into() } else { format!("_{s}") };
    }
    s
}

/// Adds `.pdf` when missing (any case).
pub fn avec_extension_pdf(nom: String) -> String {
    if nom.to_ascii_lowercase().ends_with(".pdf") { nom } else { format!("{nom}.pdf") }
}

/// Makes names unique within a set: the second `PV-12.pdf` becomes `PV-12-2.pdf`, then `-3`…
/// Returns, for each name, whether it had to be changed.
pub fn dedoublonner(noms: &mut [String]) -> Vec<bool> {
    let mut vus = std::collections::HashSet::new();
    let mut change = vec![false; noms.len()];
    for (i, nom) in noms.iter_mut().enumerate() {
        let cle = nom.to_lowercase();
        if vus.insert(cle) {
            continue;
        }
        let (base, ext) = match nom.rfind('.') {
            Some(p) if p > 0 => (nom[..p].to_string(), nom[p..].to_string()),
            _ => (nom.clone(), String::new()),
        };
        let mut n = 2;
        loop {
            let candidat = format!("{base}-{n}{ext}");
            if vus.insert(candidat.to_lowercase()) {
                *nom = candidat;
                change[i] = true;
                break;
            }
            n += 1;
        }
    }
    change
}

/// The file name of `chemin` without its extension (for naming parts).
pub fn base_du_nom(chemin: &Path) -> String {
    chemin.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "document".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_made_safe_for_windows() {
        assert_eq!(nom_sur("PV n°12 : séance/du 3?"), "PV n°12 _ séance_du 3_");
        assert_eq!(nom_sur("  "), "document");
        assert_eq!(nom_sur("fin. "), "fin");
        assert_eq!(nom_sur("CON"), "_CON");
    }

    #[test]
    fn duplicates_get_numbered_suffixes() {
        let mut noms = vec!["PV-1.pdf".to_string(), "PV-2.pdf".into(), "pv-1.pdf".into(), "PV-1.pdf".into()];
        let change = dedoublonner(&mut noms);
        assert_eq!(noms, ["PV-1.pdf", "PV-2.pdf", "pv-1-2.pdf", "PV-1-3.pdf"]);
        assert_eq!(change, [false, false, true, true]);
    }
}
