//! Page thumbnails, drawn one after the other by a background thread (the same page renderer as
//! the OCR), shown as soon as each is ready.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::sync::Arc;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

use crate::lecture::{Lecteur, MoteurOcr};

/// Width of the drawn thumbnails, in pixels (shown at half that size: sharp on high-DPI screens).
pub const LARGEUR: u32 = 192;

enum Message {
    Page(usize, u32, u32, Vec<u8>),
    Erreur(String),
}

/// The thumbnails of the open PDF.
pub struct Vignettes {
    pub chemin: PathBuf,
    pages: Vec<Option<TextureHandle>>,
    recu: Receiver<Message>,
    arret: Arc<AtomicBool>,
    pub erreur: Option<String>,
}

impl Vignettes {
    /// Starts drawing the `nombre` pages of `chemin`.
    pub fn demarrer(ctx: &egui::Context, chemin: &Path, nombre: usize) -> Vignettes {
        let (envoi, recu) = channel();
        let arret = Arc::new(AtomicBool::new(false));
        let (fin, ctx2, source) = (arret.clone(), ctx.clone(), chemin.to_path_buf());
        std::thread::spawn(move || {
            let mut lecteur = match Lecteur::ouvrir(&source, MoteurOcr::Integre) {
                Ok(l) => l,
                Err(e) => {
                    let _ = envoi.send(Message::Erreur(e.to_string()));
                    ctx2.request_repaint();
                    return;
                }
            };
            for page in 0..nombre.min(lecteur.nombre_de_pages()) {
                if fin.load(Ordering::Relaxed) {
                    return;
                }
                let message = match lecteur.vignette(page, LARGEUR) {
                    Ok((l, h, rgba)) => Message::Page(page, l, h, rgba),
                    Err(e) => Message::Erreur(e.to_string()),
                };
                if envoi.send(message).is_err() {
                    return;
                }
                ctx2.request_repaint();
            }
        });
        Vignettes { chemin: chemin.to_path_buf(), pages: vec![None; nombre], recu, arret, erreur: None }
    }

    /// Takes the thumbnails drawn since the last frame.
    pub fn recevoir(&mut self, ctx: &egui::Context) {
        while let Ok(m) = self.recu.try_recv() {
            match m {
                Message::Page(page, l, h, rgba) => {
                    let image = ColorImage::from_rgba_unmultiplied([l as usize, h as usize], &rgba);
                    let nom = format!("page-{}-{page}", self.chemin.display());
                    if let Some(place) = self.pages.get_mut(page) {
                        *place = Some(ctx.load_texture(nom, image, TextureOptions::LINEAR));
                    }
                }
                Message::Erreur(e) => self.erreur = Some(e),
            }
        }
    }

    pub fn page(&self, page: usize) -> Option<&TextureHandle> {
        self.pages.get(page).and_then(Option::as_ref)
    }

    /// Every thumbnail is drawn (or the drawing failed).
    pub fn toutes_pretes(&self) -> bool {
        self.erreur.is_some() || self.pages.iter().all(Option::is_some)
    }
}

impl Drop for Vignettes {
    fn drop(&mut self) {
        self.arret.store(true, Ordering::Relaxed);
    }
}
