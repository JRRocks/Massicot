//! The look of the validated mockup: Inter and JetBrains Mono (the fonts already shipped with
//! the engines), light panels, the icon's blue and the massicot yellow.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals,
};

pub const BLEU: Color32 = Color32::from_rgb(0x2F, 0x6F, 0xD6);
pub const BLEU_FONCE: Color32 = Color32::from_rgb(0x1B, 0x4F, 0xA8);
pub const BLEU_PALE: Color32 = Color32::from_rgb(0xDB, 0xE7, 0xFB);
pub const JAUNE: Color32 = Color32::from_rgb(0xF2, 0xB7, 0x05);
pub const JAUNE_FONCE: Color32 = Color32::from_rgb(0x8A, 0x5A, 0x00);
pub const JAUNE_PALE: Color32 = Color32::from_rgb(0xFF, 0xFB, 0xEB);
pub const VERT: Color32 = Color32::from_rgb(0x15, 0x80, 0x3D);
pub const VERT_PALE: Color32 = Color32::from_rgb(0xF0, 0xFD, 0xF4);
pub const ROUGE: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
pub const ROUGE_PALE: Color32 = Color32::from_rgb(0xFE, 0xF2, 0xF2);
pub const TEXTE: Color32 = Color32::from_rgb(0x1F, 0x29, 0x37);
pub const GRIS: Color32 = Color32::from_rgb(0x6B, 0x72, 0x80);
pub const BORDURE: Color32 = Color32::from_rgb(0xE5, 0xE7, 0xEB);
pub const FOND_OUTILS: Color32 = Color32::from_rgb(0xF7, 0xF8, 0xFA);
pub const FOND_PAGES: Color32 = Color32::from_rgb(0xEE, 0xF0, 0xF4);
pub const FOND_ETAT: Color32 = Color32::from_rgb(0xF9, 0xFA, 0xFB);

/// The family of the bold texts (Inter SemiBold).
pub fn gras() -> FontFamily {
    FontFamily::Name("gras".into())
}

/// A bold text of this size.
pub fn texte_gras(texte: impl Into<String>, taille: f32) -> egui::RichText {
    egui::RichText::new(texte).family(gras()).size(taille).color(TEXTE)
}

pub fn appliquer(ctx: &egui::Context) {
    let mut polices = FontDefinitions::empty();
    let ajouter = |polices: &mut FontDefinitions, nom: &str, octets: &'static [u8]| {
        polices.font_data.insert(nom.to_string(), Arc::new(FontData::from_static(octets)));
    };
    ajouter(&mut polices, "inter", include_bytes!("../../../../assets/fonts/Inter-Regular.ttf"));
    ajouter(&mut polices, "inter-gras", include_bytes!("../../../../assets/fonts/Inter-SemiBold.ttf"));
    ajouter(&mut polices, "mono", include_bytes!("../../../../assets/fonts/JetBrainsMono-Regular.ttf"));
    // Each family falls back on the other font for the symbols it lacks (✕, ⋮…).
    let proportionnelle = vec!["inter".to_string(), "mono".to_string()];
    let grasse = vec!["inter-gras".to_string(), "mono".to_string()];
    let fixe = vec!["mono".to_string(), "inter".to_string()];
    polices.families.insert(FontFamily::Proportional, proportionnelle);
    polices.families.insert(gras(), grasse);
    polices.families.insert(FontFamily::Monospace, fixe);
    ctx.set_fonts(polices);

    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, FontId::new(11.5, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(16.5, gras())),
            (TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(11.0, 5.0);
        style.spacing.interact_size.y = 26.0;
        style.visuals = visuels();
    });
}

fn visuels() -> Visuals {
    let mut v = Visuals::light();
    v.panel_fill = Color32::WHITE;
    v.window_fill = Color32::WHITE;
    v.extreme_bg_color = Color32::WHITE;
    v.faint_bg_color = FOND_ETAT;
    v.override_text_color = Some(TEXTE);
    v.selection.bg_fill = BLEU;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = BLEU;
    let arrondi = CornerRadius::same(7);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open, &mut v.widgets.noninteractive] {
        w.corner_radius = arrondi;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDURE);
    v.widgets.inactive.weak_bg_fill = Color32::WHITE;
    // Slider rails, check boxes, scroll bars.
    v.widgets.inactive.bg_fill = Color32::from_rgb(0xE5, 0xE7, 0xEB);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0xD1, 0xD5, 0xDB));
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xF3, 0xF4, 0xF6);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0xF3, 0xF4, 0xF6);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x9C, 0xA3, 0xAF));
    v.widgets.active.weak_bg_fill = BLEU_PALE;
    v.widgets.active.bg_fill = BLEU_PALE;
    v
}

/// The frame of a side panel or card.
pub fn cadre(fond: Color32, marge: i8) -> egui::Frame {
    egui::Frame::new().fill(fond).inner_margin(Margin::same(marge))
}

/// A bordered card.
pub fn carte(fond: Color32, bord: Color32) -> egui::Frame {
    egui::Frame::new().fill(fond).stroke(Stroke::new(1.0, bord)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::symmetric(10, 7))
}

/// A filled button (the main action, blue, or the cut, yellow).
pub fn bouton_plein(texte: &str, fond: Color32, encre: Color32) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(texte.to_string()).family(gras()).color(encre)).fill(fond).stroke(Stroke::NONE)
}
