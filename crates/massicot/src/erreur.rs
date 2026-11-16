//! Errors shown to the user, worded in French. Engine errors (English) are kept as a technical
//! detail after the French sentence.

use std::fmt;
use std::path::Path;

use pdfcraft_cos::CosError;

/// An error with a French message for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erreur(String);

pub type Resultat<T> = Result<T, Erreur>;

impl Erreur {
    pub fn new(message: impl Into<String>) -> Self {
        Erreur(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }

    /// A file could not be read or written.
    pub fn fichier(action: &str, chemin: &Path, e: &std::io::Error) -> Self {
        let raison = match e.kind() {
            std::io::ErrorKind::NotFound => "fichier ou dossier introuvable".to_string(),
            std::io::ErrorKind::PermissionDenied => "accès refusé".to_string(),
            std::io::ErrorKind::AlreadyExists => "le fichier existe déjà".to_string(),
            _ => format!("erreur système : {e}"),
        };
        Erreur(format!("impossible de {action} « {} » : {raison}", chemin.display()))
    }

    /// A PDF could not be opened.
    pub fn pdf_illisible(chemin: &Path, e: &CosError) -> Self {
        let raison = match e {
            CosError::NotPdf => "ce n'est pas un fichier PDF".to_string(),
            CosError::NeedsPassword => "le PDF est protégé par un mot de passe".to_string(),
            CosError::WrongPassword => "le mot de passe est incorrect".to_string(),
            CosError::Security(d) => format!("protection non prise en charge (détail : {d})"),
            other => format!("PDF endommagé ou illisible (détail : {other})"),
        };
        Erreur(format!("« {} » : {raison}", chemin.display()))
    }

    /// An engine reported an error (its own wording is kept as a technical detail).
    pub fn moteur(action: &str, detail: impl fmt::Display) -> Self {
        Erreur(format!("échec de {action} (détail : {detail})"))
    }
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Erreur {}
