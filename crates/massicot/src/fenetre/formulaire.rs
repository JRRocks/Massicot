//! The settings of a tool, built from the command's own definition (like its MCP schema): one
//! field per option, its French help as label. Running it goes through `outils::executer_outil`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use eframe::egui::{self, RichText};
use serde_json::{Map, Value};

use super::theme::{self, GRIS};
use crate::outils::{Genre, Outil, Parametre};

/// The value typed or chosen for one parameter.
#[derive(Debug, Clone)]
pub enum Champ {
    Case(bool),
    Texte(String),
    Liste(Vec<String>),
}

/// The settings of one tool.
pub struct Formulaire {
    pub outil: Outil,
    pub champs: Vec<(Parametre, Champ)>,
    /// The pages field follows the selection of thumbnails until the user types in it.
    pub pages_auto: bool,
}

/// The pages « 1-3,5 » (1-based) of a selection of page indexes.
pub fn plages(selection: &BTreeSet<usize>) -> String {
    let mut morceaux = Vec::new();
    let mut iter = selection.iter().copied().peekable();
    while let Some(debut) = iter.next() {
        let mut fin = debut;
        while iter.peek() == Some(&(fin + 1)) {
            fin = iter.next().unwrap_or(fin);
        }
        morceaux.push(if fin == debut { format!("{}", debut + 1) } else { format!("{}-{}", debut + 1, fin + 1) });
    }
    morceaux.join(",")
}

/// A file next to the PDF: `<name>-<suffix>.<extension>`.
pub fn a_cote(doc: &Path, suffixe: &str, extension: Option<&str>) -> String {
    let dossier = doc.parent().unwrap_or(Path::new(""));
    let nom = doc.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "document".into());
    let fichier = match extension {
        Some(ext) => format!("{nom}-{suffixe}.{ext}"),
        None => format!("{nom}-{suffixe}"),
    };
    dossier.join(fichier).to_string_lossy().into_owned()
}

impl Formulaire {
    pub fn nouveau(outil: &Outil, doc: Option<&Path>) -> Formulaire {
        let mut f = Formulaire { outil: outil.clone(), champs: Vec::new(), pages_auto: true };
        f.champs = outil
            .parametres
            .iter()
            .map(|p| {
                let champ = match p.genre {
                    Genre::Case => Champ::Case(false),
                    Genre::Fichiers | Genre::Textes => Champ::Liste(Vec::new()),
                    _ => Champ::Texte(String::new()),
                };
                (p.clone(), champ)
            })
            .collect();
        f.suivre_document(doc);
        f
    }

    /// Fills the fields that follow the open PDF: the PDF itself, the output file, the folder.
    pub fn suivre_document(&mut self, doc: Option<&Path>) {
        let Some(doc) = doc else { return };
        let chemin = doc.to_string_lossy().into_owned();
        let suffixe = suffixe_de(&self.outil.nom);
        let mut premier_fichier = true;
        for (p, champ) in &mut self.champs {
            match (&p.genre, champ) {
                (Genre::Fichier, Champ::Texte(t)) if premier_fichier => {
                    *t = chemin.clone();
                    premier_fichier = false;
                }
                (Genre::Fichiers, Champ::Liste(l)) => {
                    if l.first() != Some(&chemin) {
                        l.retain(|x| x != &chemin);
                        l.insert(0, chemin.clone());
                    }
                }
                (Genre::Sortie, Champ::Texte(t)) if p.id == "sortie" => *t = a_cote(doc, suffixe, Some("pdf")),
                (Genre::Dossier, Champ::Texte(t)) => *t = a_cote(doc, suffixe, None),
                _ => {}
            }
        }
    }

    /// The pages field follows the thumbnails selected in the middle.
    pub fn suivre_selection(&mut self, selection: &BTreeSet<usize>) {
        if !self.pages_auto {
            return;
        }
        for (p, champ) in &mut self.champs {
            if p.id == "pages"
                && let Champ::Texte(t) = champ
            {
                *t = plages(selection);
            }
        }
    }

    /// The tool's parameters, as the MCP server receives them.
    pub fn arguments(&self) -> Map<String, Value> {
        let mut args = Map::new();
        for (p, champ) in &self.champs {
            match champ {
                Champ::Case(true) => {
                    args.insert(p.id.clone(), Value::Bool(true));
                }
                Champ::Texte(t) if !t.trim().is_empty() => {
                    args.insert(p.id.clone(), Value::String(t.trim().to_string()));
                }
                Champ::Liste(l) if !l.is_empty() => {
                    args.insert(p.id.clone(), Value::Array(l.iter().map(|x| Value::String(x.clone())).collect()));
                }
                _ => {}
            }
        }
        args
    }

    /// Where the result goes: the output file's folder, or the output folder.
    pub fn dossier_du_resultat(&self) -> Option<PathBuf> {
        self.champs.iter().find_map(|(p, champ)| match (&p.genre, champ) {
            (Genre::Dossier, Champ::Texte(t)) if !t.is_empty() => Some(PathBuf::from(t)),
            (Genre::Sortie, Champ::Texte(t)) if !t.is_empty() && p.id == "sortie" => Path::new(t).parent().map(Path::to_path_buf),
            _ => None,
        })
    }

    /// The fields; true when the user changed one.
    pub fn afficher(&mut self, ui: &mut egui::Ui) -> bool {
        let mut change = false;
        let nom_outil = self.outil.nom.clone();
        let mut pages_tapees = false;
        for (p, champ) in &mut self.champs {
            match champ {
                Champ::Case(v) => {
                    change |= ui.checkbox(v, p.aide.as_str()).changed();
                }
                Champ::Texte(t) => {
                    let l = ui.label(RichText::new(p.aide.as_str()).small().color(GRIS)).id;
                    match &p.genre {
                        Genre::Choix(valeurs) => {
                            let affiche = if t.is_empty() { "par défaut".to_string() } else { t.clone() };
                            egui::ComboBox::from_id_salt((&nom_outil, &p.id))
                                .selected_text(affiche)
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    change |= ui.selectable_value(t, String::new(), "par défaut").changed();
                                    for v in valeurs {
                                        change |= ui.selectable_value(t, v.clone(), v.as_str()).changed();
                                    }
                                })
                                .response
                                .labelled_by(l);
                        }
                        Genre::Fichier | Genre::Sortie | Genre::Dossier => {
                            ui.horizontal(|ui| {
                                let r = ui.add(egui::TextEdit::singleline(t).desired_width(ui.available_width() - 42.0)).labelled_by(l);
                                change |= r.changed();
                                if ui.button("…").on_hover_text("Parcourir").clicked()
                                    && let Some(choix) = super::dialogues::choisir(&p.genre, t)
                                {
                                    *t = choix;
                                    change = true;
                                }
                            });
                        }
                        _ => {
                            let mut edit = egui::TextEdit::singleline(t).desired_width(f32::INFINITY);
                            if let Some(d) = &p.defaut {
                                edit = edit.hint_text(d.as_str());
                            }
                            let r = ui.add(edit).labelled_by(l);
                            if r.changed() {
                                change = true;
                                if p.id == "pages" {
                                    pages_tapees = true;
                                }
                            }
                        }
                    }
                }
                Champ::Liste(l) => {
                    let etiquette = ui.label(RichText::new(p.aide.as_str()).small().color(GRIS)).id;
                    let mut retirer = None;
                    for (i, x) in l.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let nom = Path::new(x).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| x.clone());
                            ui.label(format!("{}. {nom}", i + 1)).on_hover_text(x.as_str());
                            if ui.small_button("✕").on_hover_text("Retirer").clicked() {
                                retirer = Some(i);
                            }
                        });
                    }
                    if let Some(i) = retirer {
                        l.remove(i);
                        change = true;
                    }
                    if p.genre == Genre::Fichiers {
                        if ui.button("Ajouter des PDF…").clicked() {
                            l.extend(super::dialogues::plusieurs_pdf());
                            change = true;
                        }
                    } else {
                        let mut nouveau = String::new();
                        if ui.add(egui::TextEdit::singleline(&mut nouveau).hint_text("ajouter…")).labelled_by(etiquette).lost_focus() && !nouveau.trim().is_empty() {
                            l.push(nouveau.trim().to_string());
                            change = true;
                        }
                    }
                }
            }
            ui.add_space(2.0);
        }
        if pages_tapees {
            self.pages_auto = false;
        }
        change
    }
}

/// The suffix of the files a tool writes next to the PDF.
fn suffixe_de(outil: &str) -> &'static str {
    match outil {
        "fusionner" => "fusion",
        "extraire" => "extrait",
        "decouper" => "parties",
        "tourner" => "tourne",
        "supprimer" => "allege",
        "deplacer" => "reordonne",
        "inverser" => "inverse",
        "melanger" => "recto-verso",
        "ocr" => "cherchable",
        "decouper-regex" => "pv",
        _ => "resultat",
    }
}

/// The text of the button that runs a tool.
pub fn verbe(outil: &str) -> &'static str {
    match outil {
        "fusionner" => "Fusionner",
        "extraire" => "Extraire",
        "decouper" => "Découper",
        "tourner" => "Tourner",
        "supprimer" => "Supprimer les pages",
        "deplacer" => "Déplacer",
        "inverser" => "Inverser",
        "melanger" => "Mélanger",
        "texte" => "Lire le texte",
        "ocr" => "Reconnaître le texte",
        _ => "Lancer",
    }
}

/// A result line with its colour.
pub fn bandeau(ui: &mut egui::Ui, reussi: bool, texte: &str) {
    let (fond, bord, encre) = if reussi {
        (theme::VERT_PALE, egui::Color32::from_rgb(0xBB, 0xF7, 0xD0), theme::VERT)
    } else {
        (theme::ROUGE_PALE, egui::Color32::from_rgb(0xFE, 0xCA, 0xCA), theme::ROUGE)
    };
    theme::carte(fond, bord).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(texte).color(encre));
    });
}
