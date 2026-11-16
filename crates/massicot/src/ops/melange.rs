//! `melanger`: interleaves two PDFs (fronts and backs scanned separately) into one.

use std::path::Path;

use pdfcraft_cos::Document;

use crate::erreur::{Erreur, Resultat};
use crate::ops::sortie::{Ecrit, ecrire_pdf, verifier_cible};
use crate::pdf;

/// How the pages of the two files alternate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alternance {
    /// Pages taken from the first file at each turn.
    pub pas_premier: usize,
    /// Pages taken from the second file at each turn.
    pub pas_second: usize,
    /// Read the second file from its last page (backs scanned in reverse order).
    pub second_inverse: bool,
}

impl Default for Alternance {
    fn default() -> Self {
        Alternance { pas_premier: 1, pas_second: 1, second_inverse: false }
    }
}

/// The page order of the result, as (file 0 or 1, 0-based page) pairs. When one file runs out,
/// the rest of the other follows.
pub fn ordre(n1: usize, n2: usize, a: Alternance) -> Vec<(usize, usize)> {
    let second: Vec<usize> = if a.second_inverse { (0..n2).rev().collect() } else { (0..n2).collect() };
    let (p1, p2) = (a.pas_premier.max(1), a.pas_second.max(1));
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::with_capacity(n1 + n2);
    while i < n1 || j < n2 {
        for _ in 0..p1 {
            if i < n1 {
                out.push((0, i));
                i += 1;
            }
        }
        for _ in 0..p2 {
            if j < n2 {
                out.push((1, second[j]));
                j += 1;
            }
        }
    }
    out
}

/// Interleaves `premier` and `second` into `sortie`.
pub fn melanger(premier: &Path, second: &Path, alternance: Alternance, sortie: &Path, ecraser: bool) -> Resultat<Ecrit> {
    verifier_cible(sortie, ecraser)?;
    let a = pdf::ouvrir(premier)?;
    let b = pdf::ouvrir(second)?;
    let (n1, n2) = (pdf::nombre_de_pages(&a)?, pdf::nombre_de_pages(&b)?);
    let moteur = |e| Erreur::moteur("le mélange des pages", e);
    // Both files into one working document (first file's pages, then the second's), then the
    // pages taken from it in the interleaved order.
    let mut travail = Document::new_empty();
    pdfcraft_organize::import_pages(&mut travail, &a, &(0..n1).collect::<Vec<_>>(), 0).map_err(moteur)?;
    pdfcraft_organize::import_pages(&mut travail, &b, &(0..n2).collect::<Vec<_>>(), n1).map_err(moteur)?;
    let indices: Vec<usize> = ordre(n1, n2, alternance).into_iter().map(|(f, p)| if f == 0 { p } else { n1 + p }).collect();
    let mut resultat = pdfcraft_organize::extract_pages(&travail, &indices).map_err(moteur)?;
    for cle in pdfcraft_organize::INFO_KEYS {
        if let Some(v) = pdfcraft_organize::info(&a, cle) {
            let _ = pdfcraft_organize::set_info(&mut resultat, cle, &v);
        }
    }
    ecrire_pdf(&resultat, sortie, ecraser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternate_one_by_one_then_the_rest() {
        let o = ordre(3, 3, Alternance::default());
        assert_eq!(o, [(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)]);
        let o = ordre(3, 1, Alternance::default());
        assert_eq!(o, [(0, 0), (1, 0), (0, 1), (0, 2)]);
    }

    #[test]
    fn backs_scanned_in_reverse_and_steps() {
        let a = Alternance { second_inverse: true, ..Alternance::default() };
        assert_eq!(ordre(3, 3, a), [(0, 0), (1, 2), (0, 1), (1, 1), (0, 2), (1, 0)]);
        let a = Alternance { pas_premier: 2, pas_second: 1, second_inverse: false };
        assert_eq!(ordre(4, 2, a), [(0, 0), (0, 1), (1, 0), (0, 2), (0, 3), (1, 1)]);
    }
}
