//! Reading the text of pages: the PDF's own text layer, or OCR of the page picture. Every word
//! comes with its box, in points, from the top-left corner of the page as displayed (y down).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use pdfcraft_ocr::tesseract::TesseractCli;
use pdfcraft_ocr::{Line, Models, OcrImage, OcrsRecognizer, RecognizeOptions, Recognizer};
use pdfcraft_render::{PageInfo, PageRenderer, PageText, RenderConfig, RenderRequest, RequestKind};
use serde::Serialize;

use crate::erreur::{Erreur, Resultat};

/// When to read a page by OCR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ModeOcr {
    /// Only pages without a text layer.
    Auto,
    /// Every page, even with a text layer.
    Toujours,
    /// Never: pages without a text layer read as empty.
    Jamais,
}

/// Which OCR engine reads the pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum MoteurOcr {
    /// Tesseract with French when it is installed, else the built-in engine.
    Auto,
    /// The built-in engine (ocrs): no installation, but no accents.
    Integre,
    /// Tesseract (installed separately), French language.
    Tesseract,
}

/// A word and its box ([left, top, right, bottom] in points, top-left origin, y down).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Mot {
    pub texte: String,
    pub rect: [f32; 4],
    /// The line the word belongs to (words of a line follow each other).
    pub ligne: u32,
    /// OCR confidence, 0-100, when the engine gives one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confiance: Option<f32>,
}

/// Where the text of a page comes from.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    CoucheTexte,
    Ocr { moteur: String },
    Aucune,
}

/// The text of one page.
#[derive(Debug, Clone, Serialize)]
pub struct LecturePage {
    /// 1-based page number.
    pub page: usize,
    pub largeur: f32,
    pub hauteur: f32,
    pub source: Source,
    pub texte: String,
    #[serde(skip)]
    pub mots: Vec<Mot>,
    /// Mean OCR confidence (0-100) of the page, when the engine gives one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confiance: Option<f32>,
}

impl LecturePage {
    /// Whether the page has readable characters (letters or digits).
    pub fn a_du_texte(&self) -> bool {
        self.texte.chars().any(char::is_alphanumeric)
    }
}

/// The folders searched for the OCR models, in order.
pub fn dossiers_modeles() -> Vec<PathBuf> {
    let mut d = Vec::new();
    if let Some(v) = std::env::var_os("MASSICOT_MODELES") {
        d.push(PathBuf::from(v));
    }
    if let Some(exe) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        d.push(exe.join("modeles"));
    }
    // Development builds (tests) also find the models downloaded into the source tree.
    if cfg!(debug_assertions) {
        d.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../modeles"));
    }
    d
}

/// The usual places of a Tesseract installed on Windows, beyond the PATH.
fn tesseract_installe() -> Option<TesseractCli> {
    if let Some(t) = TesseractCli::find() {
        return Some(t);
    }
    let mut candidats = Vec::new();
    for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(base) = std::env::var_os(var) {
            candidats.push(PathBuf::from(&base).join("Tesseract-OCR").join("tesseract.exe"));
            candidats.push(PathBuf::from(base).join("Programs").join("Tesseract-OCR").join("tesseract.exe"));
        }
    }
    candidats.into_iter().filter(|c| c.is_file()).find_map(TesseractCli::at)
}

enum Moteur {
    Integre(Box<OcrsRecognizer>),
    Tesseract(TesseractCli),
}

impl Moteur {
    fn nom(&self) -> &'static str {
        match self {
            Moteur::Integre(_) => "integre",
            Moteur::Tesseract(_) => "tesseract",
        }
    }

    fn charger(choix: MoteurOcr) -> Resultat<Moteur> {
        let tesseract_fr = || tesseract_installe().filter(|t| t.available() && t.languages().iter().any(|(c, _)| c == "fra"));
        match choix {
            MoteurOcr::Tesseract => tesseract_fr().map(Moteur::Tesseract).ok_or_else(|| {
                Erreur::new(
                    "Tesseract avec le français est introuvable : installez-le (avec la langue « French »), \
                     ou choisissez le moteur intégré (--moteur integre)",
                )
            }),
            MoteurOcr::Auto => match tesseract_fr() {
                Some(t) => Ok(Moteur::Tesseract(t)),
                None => Moteur::charger(MoteurOcr::Integre),
            },
            MoteurOcr::Integre => {
                let dossiers = dossiers_modeles();
                let modeles = dossiers.iter().find_map(|d| Models::in_dir(d)).ok_or_else(|| {
                    Erreur::new(format!(
                        "modèles OCR introuvables : le dossier « modeles » doit être à côté de massicot.exe \
                         (cherché dans : {})",
                        dossiers.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(" ; ")
                    ))
                })?;
                let ocr = OcrsRecognizer::load(&modeles).map_err(|e| Erreur::moteur("le chargement des modèles OCR", e))?;
                Ok(Moteur::Integre(Box::new(ocr)))
            }
        }
    }

    fn reconnaitre(&self, image: &OcrImage) -> Resultat<Vec<Line>> {
        let r = match self {
            Moteur::Integre(o) => o.recognize(image, &RecognizeOptions::default()),
            Moteur::Tesseract(t) => t.recognize(image, &RecognizeOptions { language: "fr".into() }),
        };
        r.map_err(|e| Erreur::moteur("la reconnaissance de texte (OCR)", e))
    }
}

/// Reads the pages of one PDF.
pub struct Lecteur {
    chemin: PathBuf,
    rendu: PageRenderer,
    pages: Vec<PageInfo>,
    choix: MoteurOcr,
    moteur: Option<Moteur>,
    /// Resolution of the page pictures read by OCR.
    pub dpi: f32,
}

impl Lecteur {
    pub fn ouvrir(chemin: &Path, choix: MoteurOcr) -> Resultat<Lecteur> {
        let octets = std::fs::read(chemin).map_err(|e| Erreur::fichier("lire", chemin, &e))?;
        Lecteur::depuis_octets(chemin, Arc::new(octets), choix)
    }

    pub fn depuis_octets(chemin: &Path, octets: Arc<Vec<u8>>, choix: MoteurOcr) -> Resultat<Lecteur> {
        let info = pdfcraft_render::inspect(octets.clone(), None).map_err(|e| {
            Erreur::new(format!("« {} » : PDF illisible ou protégé (détail : {e:?})", chemin.display()))
        })?;
        Ok(Lecteur {
            chemin: chemin.to_path_buf(),
            rendu: PageRenderer::new(octets, RenderConfig::default()),
            pages: info.pages,
            choix,
            moteur: None,
            dpi: 300.0,
        })
    }

    pub fn nombre_de_pages(&self) -> usize {
        self.pages.len()
    }

    /// The displayed size of a page (points) and the conversion to PDF user space.
    pub fn info_page(&self, page: usize) -> &PageInfo {
        &self.pages[page]
    }

    /// The name of the OCR engine, once one has been loaded.
    pub fn moteur_utilise(&self) -> Option<&'static str> {
        self.moteur.as_ref().map(Moteur::nom)
    }

    /// The page's own text layer (visible or invisible text).
    pub fn couche_texte(&mut self, page: usize) -> Resultat<LecturePage> {
        let rendu = self.rendu.render(RenderRequest { page, kind: RequestKind::Text, scale: 1.0, ..Default::default() });
        if let Some(e) = rendu.error {
            return Err(Erreur::new(format!("page {} de « {} » illisible (détail : {e})", page + 1, self.chemin.display())));
        }
        let info = &self.pages[page];
        let mots = rendu.text.as_deref().map(mots_de).unwrap_or_default();
        let texte = rendu.text.as_deref().map(PageText::plain_text).unwrap_or_default();
        let source = if texte.chars().any(char::is_alphanumeric) { Source::CoucheTexte } else { Source::Aucune };
        Ok(LecturePage { page: page + 1, largeur: info.width, hauteur: info.height, source, texte, mots, confiance: None })
    }

    /// The page read by OCR from its picture.
    pub fn ocr(&mut self, page: usize) -> Resultat<LecturePage> {
        if self.moteur.is_none() {
            self.moteur = Some(Moteur::charger(self.choix)?);
        }
        let info = self.pages[page].clone();
        let image = self.rendu.render(RenderRequest { page, kind: RequestKind::Pixels, scale: self.dpi / 72.0, ..Default::default() });
        if let Some(e) = image.error {
            return Err(Erreur::new(format!("page {} de « {} » illisible (détail : {e})", page + 1, self.chemin.display())));
        }
        let ppp = image.width as f32 / info.width.max(1.0);
        let img = OcrImage::new(image.width, image.height, image.rgba.to_vec()).map_err(|e| Erreur::moteur("la préparation de l'image", e))?;
        let moteur = self.moteur.as_ref().ok_or_else(|| Erreur::new("moteur OCR absent"))?;
        let lignes = moteur.reconnaitre(&img)?;
        let mut mots = Vec::new();
        let mut texte = String::new();
        for (numero, ligne) in lignes.iter().enumerate() {
            let mut premiere = true;
            for w in &ligne.words {
                if w.text.trim().is_empty() {
                    continue;
                }
                if !premiere {
                    texte.push(' ');
                }
                premiere = false;
                texte.push_str(&w.text);
                mots.push(Mot {
                    texte: w.text.clone(),
                    rect: [w.rect[0] / ppp, w.rect[1] / ppp, w.rect[2] / ppp, w.rect[3] / ppp],
                    ligne: numero as u32,
                    confiance: w.confidence,
                });
            }
            if !premiere {
                texte.push('\n');
            }
        }
        let notes: Vec<f32> = mots.iter().filter_map(|m| m.confiance).collect();
        let confiance = (!notes.is_empty()).then(|| notes.iter().sum::<f32>() / notes.len() as f32);
        Ok(LecturePage {
            page: page + 1,
            largeur: info.width,
            hauteur: info.height,
            source: Source::Ocr { moteur: moteur.nom().to_string() },
            texte,
            mots,
            confiance,
        })
    }

    /// The page read as `mode` says: the text layer, OCR, or the text layer then OCR if empty.
    pub fn lire(&mut self, page: usize, mode: ModeOcr) -> Resultat<LecturePage> {
        match mode {
            ModeOcr::Toujours => self.ocr(page),
            ModeOcr::Jamais => self.couche_texte(page),
            ModeOcr::Auto => {
                let couche = self.couche_texte(page)?;
                if couche.a_du_texte() { Ok(couche) } else { self.ocr(page) }
            }
        }
    }
}

/// The words of a text layer with their boxes.
fn mots_de(t: &PageText) -> Vec<Mot> {
    let mut mots = Vec::new();
    let mut g = 0;
    while g < t.glyphs.len() {
        if t.glyphs[g].text.trim().is_empty() {
            g += 1;
            continue;
        }
        let Some((a, b)) = t.word_at(g) else { break };
        let b = b.max(a);
        let mut rect = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for gl in &t.glyphs[a..=b] {
            rect[0] = rect[0].min(gl.rect[0]);
            rect[1] = rect[1].min(gl.rect[1]);
            rect[2] = rect[2].max(gl.rect[2]);
            rect[3] = rect[3].max(gl.rect[3]);
        }
        mots.push(Mot { texte: t.text_of(a..b + 1), rect, ligne: t.line_of.get(a).copied().unwrap_or(0), confiance: None });
        g = b + 1;
    }
    mots
}
