//! Opening and saving PDF files, and page selections as the user writes them ("1-3,5,8-").

use std::path::Path;
use std::sync::Arc;

use pdfcraft_cos::{Document, SaveOptions, write_full};

use crate::erreur::{Erreur, Resultat};

/// Reads and opens a PDF file.
pub fn ouvrir(chemin: &Path) -> Resultat<Document> {
    let octets = std::fs::read(chemin).map_err(|e| Erreur::fichier("lire", chemin, &e))?;
    ouvrir_octets(chemin, octets)
}

/// Opens PDF bytes read from `chemin` (named in error messages).
pub fn ouvrir_octets(chemin: &Path, octets: Vec<u8>) -> Resultat<Document> {
    Document::open(Arc::new(octets)).map_err(|e| Erreur::pdf_illisible(chemin, &e))
}

/// The bytes of a full rewrite of `doc` (compressed object streams).
pub fn octets(doc: &Document) -> Resultat<Vec<u8>> {
    let options = SaveOptions { mod_date: Some(pdfcraft_cos::pdf_date(maintenant())), ..SaveOptions::default() };
    write_full(doc, &options).map_err(|e| Erreur::moteur("l'écriture du PDF", e))
}

/// Writes `doc` to `chemin` (full rewrite), creating the parent folder if needed.
/// Returns the size written, in bytes.
pub fn enregistrer(doc: &Document, chemin: &Path) -> Resultat<u64> {
    let octets = octets(doc)?;
    ecrire(chemin, &octets)?;
    Ok(octets.len() as u64)
}

/// Writes bytes to a file, creating its folder if needed.
pub fn ecrire(chemin: &Path, octets: &[u8]) -> Resultat<()> {
    if let Some(dossier) = chemin.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dossier).map_err(|e| Erreur::fichier("créer le dossier", dossier, &e))?;
    }
    std::fs::write(chemin, octets).map_err(|e| Erreur::fichier("écrire", chemin, &e))
}

fn maintenant() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// The number of pages of `doc`.
pub fn nombre_de_pages(doc: &Document) -> Resultat<usize> {
    pdfcraft_organize::page_count(doc).map_err(|e| Erreur::moteur("la lecture des pages", e))
}

/// Parses a page selection such as `1-3,5,8-` (1-based; `N-` runs to the last page, `-N` starts
/// at the first one, `fin` is the last page) into 0-based indices, in the order written.
pub fn pages(selection: &str, nombre: usize) -> Resultat<Vec<usize>> {
    let invalide = |detail: String| Erreur::new(format!("sélection de pages « {selection} » invalide : {detail}"));
    let numero = |s: &str| -> Resultat<usize> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("fin") {
            return Ok(nombre);
        }
        match s.parse::<usize>() {
            Ok(0) => Err(invalide("les pages se comptent à partir de 1".into())),
            Ok(n) if n > nombre => Err(invalide(format!("la page {n} n'existe pas (le document a {nombre} pages)"))),
            Ok(n) => Ok(n),
            Err(_) => Err(invalide(format!("« {s} » n'est pas un numéro de page"))),
        }
    };
    let mut out = Vec::new();
    for morceau in selection.split(',') {
        let morceau = morceau.trim();
        if morceau.is_empty() {
            return Err(invalide("élément vide".into()));
        }
        match morceau.split_once('-') {
            None => out.push(numero(morceau)? - 1),
            Some((a, b)) => {
                let debut = if a.trim().is_empty() { 1 } else { numero(a)? };
                let fin = if b.trim().is_empty() { nombre } else { numero(b)? };
                if debut <= fin {
                    out.extend(debut - 1..fin);
                } else {
                    out.extend((fin - 1..debut).rev());
                }
            }
        }
    }
    if out.is_empty() {
        return Err(invalide("aucune page".into()));
    }
    Ok(out)
}

/// Writes 0-based page indices as a short 1-based selection ("1-3, 5").
pub fn resume_pages(indices: &[usize]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < indices.len() {
        let mut j = i;
        while j + 1 < indices.len() && indices[j + 1] == indices[j] + 1 {
            j += 1;
        }
        if j > i {
            parts.push(format!("{}-{}", indices[i] + 1, indices[j] + 1));
        } else {
            parts.push(format!("{}", indices[i] + 1));
        }
        i = j + 1;
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selections_follow_the_written_order() {
        assert_eq!(pages("1-3,5", 10).unwrap(), vec![0, 1, 2, 4]);
        assert_eq!(pages("5,1-2", 10).unwrap(), vec![4, 0, 1]);
        assert_eq!(pages("8-", 10).unwrap(), vec![7, 8, 9]);
        assert_eq!(pages("-2", 10).unwrap(), vec![0, 1]);
        assert_eq!(pages("3-1", 10).unwrap(), vec![2, 1, 0]);
        assert_eq!(pages("fin", 10).unwrap(), vec![9]);
        assert_eq!(pages(" 2 , 4 ", 10).unwrap(), vec![1, 3]);
    }

    #[test]
    fn bad_selections_are_explained_in_french() {
        assert!(pages("0", 5).unwrap_err().message().contains("à partir de 1"));
        assert!(pages("6", 5).unwrap_err().message().contains("la page 6 n'existe pas"));
        assert!(pages("a", 5).unwrap_err().message().contains("n'est pas un numéro"));
        assert!(pages("1,,2", 5).unwrap_err().message().contains("élément vide"));
    }

    #[test]
    fn summaries_merge_runs() {
        assert_eq!(resume_pages(&[0, 1, 2, 4, 6, 7]), "1-3, 5, 7-8");
        assert_eq!(resume_pages(&[3]), "4");
    }
}
