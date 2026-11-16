//! Invented test PDFs for Massicot: pages of text drawn in Helvetica (WinAnsi, so accents work),
//! with metadata and bookmarks. Nothing here comes from a real document.

use pdfcraft_cos::{Dict, Document, ObjRef, Object, PdfString, SaveOptions, Stream, write_full};

/// A4 portrait, in points.
pub const A4: (f64, f64) = (595.0, 842.0);

/// A line of text: position of the baseline from the bottom-left corner, font size, text.
#[derive(Clone, Debug)]
pub struct Ligne {
    pub x: f64,
    pub y: f64,
    pub taille: f64,
    pub texte: String,
}

impl Ligne {
    pub fn new(x: f64, y: f64, taille: f64, texte: impl Into<String>) -> Self {
        Ligne { x, y, taille, texte: texte.into() }
    }
}

/// One page: its size in points and its lines.
#[derive(Clone, Debug)]
pub struct Page {
    pub largeur: f64,
    pub hauteur: f64,
    pub lignes: Vec<Ligne>,
}

impl Page {
    pub fn a4(lignes: Vec<Ligne>) -> Self {
        Page { largeur: A4.0, hauteur: A4.1, lignes }
    }
}

/// A document made of these pages, with a title and one bookmark per `(title, 0-based page)`.
pub fn document(pages: &[Page], titre: Option<&str>, signets: &[(&str, usize)]) -> Document {
    let mut doc = Document::new_empty();
    let font = police_helvetica(&mut doc);
    for (i, p) in pages.iter().enumerate() {
        let page = pdfcraft_organize::insert_blank_page(&mut doc, i, p.largeur, p.hauteur).expect("page tree");
        if !p.lignes.is_empty() {
            let contenu = flux_texte(&p.lignes);
            let flux = doc.add(Object::Stream(Stream::flate(Dict::new(), &contenu)));
            let mut polices = Dict::new();
            polices.set(b"F1".to_vec(), Object::Ref(font));
            let mut ressources = Dict::new();
            ressources.set(b"Font".to_vec(), Object::Dict(polices));
            doc.update_dict(page, |d| {
                d.set(b"Contents".to_vec(), Object::Ref(flux));
                d.set(b"Resources".to_vec(), Object::Dict(ressources));
            })
            .expect("page dict");
        }
    }
    if let Some(t) = titre {
        pdfcraft_organize::set_info(&mut doc, "Title", t).expect("info");
    }
    for (n, (titre, page)) in signets.iter().enumerate() {
        pdfcraft_organize::add_bookmark(&mut doc, &[], n, titre, *page).expect("bookmark");
    }
    doc
}

fn police_helvetica(doc: &mut Document) -> ObjRef {
    let mut f = Dict::new();
    f.set(b"Type".to_vec(), Object::name("Font"));
    f.set(b"Subtype".to_vec(), Object::name("Type1"));
    f.set(b"BaseFont".to_vec(), Object::name("Helvetica"));
    f.set(b"Encoding".to_vec(), Object::name("WinAnsiEncoding"));
    doc.add(Object::Dict(f))
}

/// The content stream drawing `lignes` in Helvetica.
pub fn flux_texte(lignes: &[Ligne]) -> Vec<u8> {
    let mut s = Vec::new();
    for l in lignes {
        s.extend_from_slice(format!("BT /F1 {} Tf 1 0 0 1 {} {} Tm ", l.taille, l.x, l.y).as_bytes());
        let mut obj = Vec::new();
        pdfcraft_cos::serialize(&Object::String(PdfString::literal(pdfcraft_fonts::win_ansi(&l.texte))), &mut obj);
        s.extend_from_slice(&obj);
        s.extend_from_slice(b" Tj ET\n");
    }
    s
}

/// The bytes of `doc`, ready to write.
pub fn octets(doc: &Document) -> Vec<u8> {
    write_full(doc, &SaveOptions::default()).expect("PDF writer")
}
