//! `fusionner`: several PDFs into one, in the order given.

use std::path::{Path, PathBuf};

use pdfcraft_cos::Object;

use crate::erreur::{Erreur, Resultat};
use crate::ops::sortie::{Ecrit, base_du_nom, ecrire_pdf, verifier_cible};
use crate::pdf;

/// Merges `entrees` into `sortie`. Each file gets a bookmark (its name) over its own bookmarks,
/// unless `sans_signets`, which leaves the result without any bookmark.
pub fn fusionner(entrees: &[PathBuf], sortie: &Path, sans_signets: bool, ecraser: bool) -> Resultat<Ecrit> {
    if entrees.len() < 2 {
        return Err(Erreur::new("la fusion demande au moins deux fichiers"));
    }
    verifier_cible(sortie, ecraser)?;
    let mut fusion = pdfcraft_organize::Combiner::new();
    for chemin in entrees {
        let doc = pdf::ouvrir(chemin)?;
        fusion
            .add(&base_du_nom(chemin), &doc, None)
            .map_err(|e| Erreur::moteur(&format!("l'ajout de « {} »", chemin.display()), e))?;
    }
    let mut doc = fusion.finish().map_err(|e| Erreur::moteur("la fusion", e))?;
    if sans_signets {
        retirer_signets(&mut doc)?;
    }
    ecrire_pdf(&doc, sortie, ecraser)
}

/// Drops the outline (all bookmarks) of `doc`.
pub fn retirer_signets(doc: &mut pdfcraft_cos::Document) -> Resultat<()> {
    let Some(catalogue) = doc.root() else { return Ok(()) };
    doc.update_dict(catalogue, |d| {
        d.remove(b"Outlines");
        if d.name(b"PageMode") == Some(b"UseOutlines".as_slice()) {
            d.set(b"PageMode".to_vec(), Object::name("UseNone"));
        }
    })
    .map_err(|e| Erreur::moteur("le retrait des signets", e))
}
