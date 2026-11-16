//! Page operations on one document: extract, rotate, delete, move, reverse.

use std::path::Path;

use crate::erreur::{Erreur, Resultat};
use crate::ops::sortie::{Ecrit, ecrire_pdf, verifier_cible};
use crate::pdf;

/// Which pages a command applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Toutes,
    /// A selection as the user writes it ("1-3,5").
    Pages(String),
    /// Pages 2, 4, 6…
    Paires,
    /// Pages 1, 3, 5…
    Impaires,
}

impl Selection {
    /// The 0-based indices selected in a document of `nombre` pages, in order.
    pub fn indices(&self, nombre: usize) -> Resultat<Vec<usize>> {
        Ok(match self {
            Selection::Toutes => (0..nombre).collect(),
            Selection::Pages(s) => pdf::pages(s, nombre)?,
            Selection::Paires => (0..nombre).filter(|i| i % 2 == 1).collect(),
            Selection::Impaires => (0..nombre).filter(|i| i % 2 == 0).collect(),
        })
    }
}

fn moteur(action: &str) -> impl Fn(pdfcraft_organize::OrganizeError) -> Erreur + '_ {
    move |e| Erreur::moteur(action, e)
}

/// A new PDF with the selected pages, in the order given (a page may appear twice).
pub fn extraire(entree: &Path, pages: &str, sortie: &Path, ecraser: bool) -> Resultat<Ecrit> {
    verifier_cible(sortie, ecraser)?;
    let doc = pdf::ouvrir(entree)?;
    let n = pdf::nombre_de_pages(&doc)?;
    let indices = pdf::pages(pages, n)?;
    let extrait = pdfcraft_organize::extract_pages(&doc, &indices).map_err(moteur("l'extraction des pages"))?;
    ecrire_pdf(&extrait, sortie, ecraser)
}

/// Rotates the selected pages clockwise by `angle` degrees (a multiple of 90, negative turns left).
pub fn tourner(entree: &Path, angle: i64, selection: &Selection, sortie: &Path, ecraser: bool) -> Resultat<Ecrit> {
    if angle % 90 != 0 {
        return Err(Erreur::new(format!("angle {angle}° impossible : choisissez 90, 180, 270 ou -90")));
    }
    verifier_cible(sortie, ecraser)?;
    let mut doc = pdf::ouvrir(entree)?;
    let indices = selection.indices(pdf::nombre_de_pages(&doc)?)?;
    if indices.is_empty() {
        return Err(Erreur::new("aucune page ne correspond à la sélection"));
    }
    pdfcraft_organize::rotate_pages(&mut doc, &indices, angle).map_err(moteur("la rotation"))?;
    ecrire_pdf(&doc, sortie, ecraser)
}

/// Removes the selected pages.
pub fn supprimer(entree: &Path, selection: &Selection, sortie: &Path, ecraser: bool) -> Resultat<Ecrit> {
    verifier_cible(sortie, ecraser)?;
    let mut doc = pdf::ouvrir(entree)?;
    let n = pdf::nombre_de_pages(&doc)?;
    let mut indices = selection.indices(n)?;
    indices.sort_unstable();
    indices.dedup();
    if indices.len() >= n {
        return Err(Erreur::new("impossible de supprimer toutes les pages : il doit en rester au moins une"));
    }
    pdfcraft_organize::delete_pages(&mut doc, &indices).map_err(moteur("la suppression des pages"))?;
    ecrire_pdf(&doc, sortie, ecraser)
}

/// Where moved pages go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// They start at this 1-based position of the result.
    A(usize),
    Debut,
    Fin,
}

/// Moves the selected pages (kept in their relative order) to `position`.
pub fn deplacer(entree: &Path, pages: &str, position: Position, sortie: &Path, ecraser: bool) -> Resultat<Ecrit> {
    verifier_cible(sortie, ecraser)?;
    let mut doc = pdf::ouvrir(entree)?;
    let n = pdf::nombre_de_pages(&doc)?;
    let mut indices = pdf::pages(pages, n)?;
    indices.sort_unstable();
    indices.dedup();
    let restantes = n - indices.len();
    let a = match position {
        Position::Debut => 0,
        Position::Fin => restantes,
        Position::A(p) if p >= 1 && p <= restantes + 1 => p - 1,
        Position::A(p) => {
            return Err(Erreur::new(format!(
                "position {p} impossible : entre 1 et {} (le document garde {restantes} autres pages)",
                restantes + 1
            )));
        }
    };
    pdfcraft_organize::move_pages(&mut doc, &indices, a).map_err(moteur("le déplacement des pages"))?;
    ecrire_pdf(&doc, sortie, ecraser)
}

/// Reverses the order of the selected pages (all by default) among the places they occupy.
pub fn inverser(entree: &Path, selection: &Selection, sortie: &Path, ecraser: bool) -> Resultat<Ecrit> {
    verifier_cible(sortie, ecraser)?;
    let mut doc = pdf::ouvrir(entree)?;
    let indices = selection.indices(pdf::nombre_de_pages(&doc)?)?;
    let mut distinctes = indices.clone();
    distinctes.sort_unstable();
    distinctes.dedup();
    if distinctes.len() >= 2 {
        pdfcraft_organize::reverse_pages(&mut doc, &distinctes).map_err(moteur("l'inversion des pages"))?;
    }
    ecrire_pdf(&doc, sortie, ecraser)
}
