//! The cut at each header, in four steps as on the validated mockup: the PDF, the header, the dry
//! run report (one block per document, its pages, the header read, the alerts), then the cut.

use std::path::{Path, PathBuf};

use eframe::egui::{self, RichText};
use serde_json::{Map, Value, json};

use super::formulaire::{a_cote, bandeau};
use super::theme::{self, GRIS, JAUNE, JAUNE_FONCE, JAUNE_PALE, BLEU, BLEU_PALE, BORDURE};

/// Ready-made headers: (name in the list, expression, file names).
pub const MODELES: [(&str, &str, &str); 3] = [
    ("Procès-verbal n° …", r"PROCÈS-VERBAL N° (?<num>\d+)", "PV-{num}.pdf"),
    ("Délibération n° …", r"DÉLIBÉRATION N° (?<num>\d+)", "Deliberation-{num}.pdf"),
    ("Autre (à saisir)", "", "{bloc}.pdf"),
];

pub struct Decoupe {
    /// 1 PDF, 2 header, 3 report, 4 cut.
    pub etape: u8,
    pub modele: usize,
    pub regex: String,
    pub zone_haute: u8,
    pub nom: String,
    pub dossier: String,
    /// The dry run report (JSON of `decouper-regex`).
    pub rapport: Option<Value>,
    /// The report of the cut.
    pub fin: Option<Value>,
    pub erreur: Option<String>,
}

/// What the panel asks the window to do.
pub enum Demande {
    Rien,
    /// Run `decouper-regex` with these parameters (dry run, or the cut with its token).
    Lancer(Map<String, Value>),
    OuvrirDossier(PathBuf),
}

impl Default for Decoupe {
    fn default() -> Self {
        Decoupe {
            etape: 1,
            modele: 0,
            regex: MODELES[0].1.to_string(),
            zone_haute: 30,
            nom: MODELES[0].2.to_string(),
            dossier: String::new(),
            rapport: None,
            fin: None,
            erreur: None,
        }
    }
}

fn etapes(ui: &mut egui::Ui, courante: u8) {
    ui.horizontal(|ui| {
        let largeur = (ui.available_width() - 3.0 * 4.0) / 4.0;
        for (n, nom) in ["1 PDF", "2 En-tête", "3 Rapport", "4 Couper"].iter().enumerate() {
            let n = n as u8 + 1;
            let (fond, encre) = match n.cmp(&courante) {
                std::cmp::Ordering::Less => (BLEU_PALE, theme::BLEU_FONCE),
                std::cmp::Ordering::Equal => (BLEU, egui::Color32::WHITE),
                std::cmp::Ordering::Greater => (egui::Color32::from_rgb(0xF3, 0xF4, 0xF6), GRIS),
            };
            egui::Frame::new().fill(fond).corner_radius(5).inner_margin(egui::Margin::symmetric(0, 4)).show(ui, |ui| {
                ui.set_width(largeur);
                ui.vertical_centered(|ui| ui.label(RichText::new(*nom).size(11.0).color(encre)));
            });
        }
    });
}

impl Decoupe {
    /// The PDF changed: back to step 1, and the output folder next to it.
    pub fn suivre_document(&mut self, doc: Option<&Path>) {
        *self = Decoupe { modele: self.modele, regex: self.regex.clone(), zone_haute: self.zone_haute, nom: self.nom.clone(), ..Decoupe::default() };
        if let Some(doc) = doc {
            self.dossier = a_cote(doc, "pv", None);
        }
    }

    fn parametres(&self, doc: &Path) -> Map<String, Value> {
        let mut args = Map::new();
        args.insert("fichier".into(), json!(doc.to_string_lossy()));
        args.insert("regex".into(), json!([self.regex.trim()]));
        args.insert("zone_haute".into(), json!(self.zone_haute.to_string()));
        args.insert("modele".into(), json!(self.nom.trim()));
        args.insert("dossier".into(), json!(self.dossier.trim()));
        args
    }

    /// The result of a run of `decouper-regex`.
    pub fn recevoir(&mut self, resultat: Result<Value, String>) {
        match resultat {
            Ok(rapport) if rapport["a_blanc"].as_bool() == Some(false) => {
                self.fin = Some(rapport);
                self.etape = 4;
                self.erreur = None;
            }
            Ok(rapport) => {
                self.rapport = Some(rapport);
                self.etape = 3;
                self.erreur = None;
            }
            Err(e) => self.erreur = Some(e),
        }
    }

    /// The blocks of the report: (file, first page, last page, alerts?), pages 1-based.
    pub fn blocs(&self) -> Vec<(String, usize, usize, bool)> {
        let Some(r) = self.rapport.as_ref().filter(|_| self.etape >= 3) else { return Vec::new() };
        r["blocs"]
            .as_array()
            .map(|b| {
                b.iter()
                    .map(|b| {
                        let alerte = b["alertes"].as_array().is_some_and(|a| !a.is_empty());
                        (
                            b["fichier"].as_str().unwrap_or("").to_string(),
                            b["premiere_page"].as_u64().unwrap_or(1) as usize,
                            b["derniere_page"].as_u64().unwrap_or(1) as usize,
                            alerte,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The right-hand panel.
    pub fn afficher(&mut self, ui: &mut egui::Ui, doc: Option<&Path>, occupe: bool) -> Demande {
        let mut demande = Demande::Rien;
        ui.heading("Découpe par en-tête");
        etapes(ui, self.etape);
        ui.add_space(4.0);
        let Some(doc) = doc else {
            ui.label("Un fichier par document : chaque page qui porte l'en-tête ouvre un nouveau fichier.");
            ui.label(RichText::new("Ouvrez d'abord le PDF à découper (bouton « Ouvrir un PDF… » ou glisser-déposer).").color(GRIS));
            return demande;
        };
        if let Some(e) = &self.erreur {
            bandeau(ui, false, e);
        }
        match self.etape {
            1 => {
                ui.label("Un fichier par document : chaque page qui porte l'en-tête ouvre un nouveau fichier.");
                ui.label(RichText::new("PDF à découper").small().color(GRIS));
                theme::carte(egui::Color32::WHITE, BORDURE).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(doc.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
                });
                ui.label(RichText::new("Les pages scannées sont lues par l'OCR (1 à 3 secondes par page).").small().color(GRIS));
                bas(ui, |ui| {
                    if ui.add(theme::bouton_plein("Suivant ›", BLEU, egui::Color32::WHITE)).clicked() {
                        self.etape = 2;
                    }
                });
            }
            2 => {
                let l = ui.label(RichText::new("Modèle").small().color(GRIS));
                let avant = self.modele;
                egui::ComboBox::from_id_salt("modele-en-tete")
                    .selected_text(MODELES[self.modele].0)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for (i, m) in MODELES.iter().enumerate() {
                            ui.selectable_value(&mut self.modele, i, m.0);
                        }
                    })
                    .response
                    .labelled_by(l.id);
                if self.modele != avant {
                    self.regex = MODELES[self.modele].1.to_string();
                    self.nom = MODELES[self.modele].2.to_string();
                }
                let l = ui.label(RichText::new("Expression cherchée").small().color(GRIS));
                ui.add(egui::TextEdit::singleline(&mut self.regex).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY)).labelled_by(l.id);
                ui.label(RichText::new("Majuscules et accents ignorés.").small().color(GRIS));
                let l = ui.label(RichText::new(format!("Chercher dans le haut de la page : {} %", self.zone_haute)).small().color(GRIS));
                ui.spacing_mut().slider_width = ui.available_width();
                ui.add(egui::Slider::new(&mut self.zone_haute, 5..=100).show_value(false).trailing_fill(true)).labelled_by(l.id);
                let l = ui.label(RichText::new("Nom des fichiers").small().color(GRIS));
                ui.add(egui::TextEdit::singleline(&mut self.nom).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY)).labelled_by(l.id);
                let l = ui.label(RichText::new("Dossier").small().color(GRIS));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.dossier).desired_width(ui.available_width() - 42.0)).labelled_by(l.id);
                    if ui.button("…").on_hover_text("Parcourir").clicked()
                        && let Some(d) = super::dialogues::choisir(&crate::outils::Genre::Dossier, &self.dossier)
                    {
                        self.dossier = d;
                    }
                });
                // The long button gets its own line, « Retour » below it.
                let reste = ui.available_height() - 74.0;
                if reste > 0.0 {
                    ui.add_space(reste);
                }
                let pret = !occupe && !self.regex.trim().is_empty() && !self.dossier.trim().is_empty();
                let texte = if occupe { "Lecture des pages…" } else { "Voir le rapport (rien n'est écrit)" };
                if ui.add_enabled(pret, theme::bouton_plein(texte, BLEU, egui::Color32::WHITE).min_size(egui::vec2(ui.available_width(), 32.0))).clicked() {
                    demande = Demande::Lancer(self.parametres(doc));
                }
                if ui.button("‹ Retour").clicked() {
                    self.etape = 1;
                }
            }
            3 => {
                let Some(rapport) = self.rapport.clone() else { return demande };
                let blocs = rapport["blocs"].as_array().cloned().unwrap_or_default();
                let alertes = blocs.iter().filter(|b| b["alertes"].as_array().is_some_and(|a| !a.is_empty())).count();
                ui.label(texte_compte(blocs.len(), alertes));
                egui::ScrollArea::vertical().max_height(ui.available_height() - 50.0).auto_shrink([false, true]).show(ui, |ui| {
                    for b in &blocs {
                        carte_bloc(ui, b);
                    }
                    for a in rapport["alertes"].as_array().into_iter().flatten() {
                        ui.label(RichText::new(format!("⚠ {}", a.as_str().unwrap_or(""))).color(JAUNE_FONCE));
                    }
                });
                bas(ui, |ui| {
                    if ui.button("‹ Modifier").clicked() {
                        self.etape = 2;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let texte = if occupe { "Découpe en cours…".to_string() } else { format!("Couper en {} fichiers", blocs.len()) };
                        if ui.add_enabled(!occupe && !blocs.is_empty(), theme::bouton_plein(&texte, JAUNE, theme::TEXTE)).clicked() {
                            let mut args = self.parametres(doc);
                            args.insert("couper".into(), rapport["jeton"].clone());
                            demande = Demande::Lancer(args);
                        }
                    });
                });
            }
            _ => {
                let fin = self.fin.clone().unwrap_or(Value::Null);
                let n = fin["blocs"].as_array().map(Vec::len).unwrap_or(0);
                let dossier = fin["dossier"].as_str().map(str::to_string).unwrap_or_else(|| self.dossier.clone());
                bandeau(ui, true, &format!("✓ {n} fichiers écrits dans {dossier}, avec le rapport (rapport-decoupe.json)."));
                if ui.button("Ouvrir le dossier").clicked() {
                    demande = Demande::OuvrirDossier(PathBuf::from(&dossier));
                }
                if ui.button("Nouvelle découpe").clicked() {
                    self.suivre_document(Some(doc));
                }
                ui.label(RichText::new("Aucun fichier existant n'a été remplacé.").small().color(GRIS));
            }
        }
        demande
    }
}

fn texte_compte(blocs: usize, alertes: usize) -> String {
    let b = if blocs > 1 { format!("{blocs} documents trouvés") } else { format!("{blocs} document trouvé") };
    match alertes {
        0 => b,
        1 => format!("{b} · 1 alerte"),
        n => format!("{b} · {n} alertes"),
    }
}

/// One block of the report: its file, pages, header read, alerts.
fn carte_bloc(ui: &mut egui::Ui, b: &Value) {
    let alertes: Vec<&str> = b["alertes"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    let (fond, bord) = if alertes.is_empty() { (egui::Color32::WHITE, BORDURE) } else { (JAUNE_PALE, egui::Color32::from_rgb(0xE3, 0xA9, 0x00)) };
    theme::carte(fond, bord).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let (p1, p2) = (b["premiere_page"].as_u64().unwrap_or(0), b["derniere_page"].as_u64().unwrap_or(0));
        let pages = if p1 == p2 { format!("page {p1}") } else { format!("pages {p1} à {p2}") };
        ui.horizontal_wrapped(|ui| {
            ui.label(theme::texte_gras(b["fichier"].as_str().unwrap_or(""), 13.0));
            ui.label(format!("· {pages}"));
        });
        if let Some(t) = b["en_tete"]["texte_trouve"].as_str() {
            let confiance = b["en_tete"]["confiance"].as_f64().map(|c| format!(" · {c:.0} %")).unwrap_or_default();
            ui.label(RichText::new(format!("« {t} »{confiance}")).color(GRIS));
        }
        for a in alertes {
            ui.label(RichText::new(format!("⚠ {a}")).color(JAUNE_FONCE));
        }
    });
}

/// The buttons at the bottom of the panel.
fn bas(ui: &mut egui::Ui, contenu: impl FnOnce(&mut egui::Ui)) {
    let reste = ui.available_height() - 34.0;
    if reste > 0.0 {
        ui.add_space(reste);
    }
    ui.horizontal(contenu);
}
