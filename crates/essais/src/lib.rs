//! Invented test PDFs for Massicot: pages of text drawn in Helvetica (WinAnsi, so accents work),
//! with metadata and bookmarks. Nothing here comes from a real document.

use std::sync::Arc;

use pdfcraft_cos::{Dict, Document, ObjRef, Object, PdfString, SaveOptions, Stream, write_full};
use pdfcraft_render::{PageRenderer, RenderConfig, RenderRequest, RequestKind};

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

/// Scanner settings: resolution, speckle noise (dots per 10,000 pixels), slight skew (degrees).
#[derive(Clone, Copy, Debug)]
pub struct Scan {
    pub dpi: f32,
    pub bruit: u32,
    pub inclinaison: f32,
    pub graine: u64,
}

impl Default for Scan {
    fn default() -> Self {
        Scan { dpi: 200.0, bruit: 30, inclinaison: 0.0, graine: 1 }
    }
}

/// A "scanned" copy of a PDF: each page becomes a greyscale picture of itself (no text layer),
/// with speckles and an optional skew, like a page through a scanner.
pub fn scanner(pdf: &[u8], reglage: Scan) -> Document {
    let mut rendu = PageRenderer::new(Arc::new(pdf.to_vec()), RenderConfig::default());
    let mut doc = Document::new_empty();
    let mut graine = reglage.graine.max(1);
    for i in 0..rendu.page_count() {
        let p = rendu.render(RenderRequest { page: i, kind: RequestKind::Pixels, scale: reglage.dpi / 72.0, ..Default::default() });
        assert!(p.error.is_none(), "rendu de la page {i}: {:?}", p.error);
        let (l, h) = (p.width as usize, p.height as usize);
        let rgba = p.rgba.to_vec();
        let mut gris: Vec<u8> = rgba.chunks_exact(4).map(|c| ((c[0] as u32 * 299 + c[1] as u32 * 587 + c[2] as u32 * 114) / 1000) as u8).collect();
        if reglage.inclinaison != 0.0 {
            gris = incliner(&gris, l, h, reglage.inclinaison);
        }
        for v in gris.iter_mut() {
            graine = graine.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let tirage = (graine >> 33) % 10_000;
            if (tirage as u32) < reglage.bruit {
                *v = if graine & 1 == 0 { 40 } else { 200 };
            } else {
                // A paper that is not perfectly white.
                *v = v.saturating_sub(((graine >> 20) % 12) as u8);
            }
        }
        let (largeur_pt, hauteur_pt) = (l as f64 * 72.0 / reglage.dpi as f64, h as f64 * 72.0 / reglage.dpi as f64);
        let page = pdfcraft_organize::insert_blank_page(&mut doc, i, largeur_pt, hauteur_pt).expect("page tree");
        let mut d = Dict::new();
        d.set(b"Type".to_vec(), Object::name("XObject"));
        d.set(b"Subtype".to_vec(), Object::name("Image"));
        d.set(b"Width".to_vec(), Object::Int(l as i64));
        d.set(b"Height".to_vec(), Object::Int(h as i64));
        d.set(b"ColorSpace".to_vec(), Object::name("DeviceGray"));
        d.set(b"BitsPerComponent".to_vec(), Object::Int(8));
        let image = doc.add(Object::Stream(Stream::flate(d, &gris)));
        let contenu = format!("q {largeur_pt:.3} 0 0 {hauteur_pt:.3} 0 0 cm /Scan Do Q\n");
        let flux = doc.add(Object::Stream(Stream::flate(Dict::new(), contenu.as_bytes())));
        let mut xobjets = Dict::new();
        xobjets.set(b"Scan".to_vec(), Object::Ref(image));
        let mut ressources = Dict::new();
        ressources.set(b"XObject".to_vec(), Object::Dict(xobjets));
        doc.update_dict(page, |d| {
            d.set(b"Contents".to_vec(), Object::Ref(flux));
            d.set(b"Resources".to_vec(), Object::Dict(ressources));
        })
        .expect("page dict");
    }
    doc
}

/// Rotates a greyscale picture by `degres` around its centre (white outside).
fn incliner(gris: &[u8], l: usize, h: usize, degres: f32) -> Vec<u8> {
    let (s, c) = degres.to_radians().sin_cos();
    let (cx, cy) = (l as f32 / 2.0, h as f32 / 2.0);
    let mut out = vec![255u8; gris.len()];
    for y in 0..h {
        for x in 0..l {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            let (sx, sy) = (c * dx + s * dy + cx, -s * dx + c * dy + cy);
            if sx >= 0.0 && sy >= 0.0 && (sx as usize) < l && (sy as usize) < h {
                out[y * l + x] = gris[sy as usize * l + sx as usize];
            }
        }
    }
    out
}
