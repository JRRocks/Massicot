//! The window (`massicot fenetre`), as on the validated mockup (variant 1 « Atelier », D-27): the
//! tools on the left, the pages of the open PDF in the middle, the tool's settings on the right.
//! Every action runs the tool through `outils::executer_outil`, as the command line and MCP do.

mod decoupe;
mod dialogues;
mod formulaire;
pub mod theme;
mod vignettes;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, StrokeKind};
use serde_json::{Map, Value, json};

use crate::erreur::{Erreur, Resultat};
use crate::outils::{self, Outil};
use decoupe::{Decoupe, Demande};
use formulaire::{Formulaire, bandeau};
use theme::{BLEU, BORDURE, FOND_ETAT, FOND_OUTILS, FOND_PAGES, GRIS, TEXTE};
use vignettes::Vignettes;

/// The tools in the left column: group, then (command, name, symbol).
const GROUPES: [(&str, &[(&str, &str, &str)]); 4] = [
    ("Assembler", &[("fusionner", "Fusionner", "⊕"), ("melanger", "Mélanger recto/verso", "↔")]),
    ("Découper", &[("decouper", "Découper", "÷"), ("decouper-regex", "Découpe par en-tête", "¶"), ("extraire", "Extraire des pages", "↗")]),
    ("Pages", &[("tourner", "Tourner", "↻"), ("supprimer", "Supprimer", "✕"), ("deplacer", "Déplacer", "↕"), ("inverser", "Inverser", "↑↓")]),
    ("Texte", &[("ocr", "OCR", "A"), ("texte", "Texte des pages", "T"), ("infos", "Infos", "i")]),
];

const LARGEUR_VIGNETTE: f32 = 96.0;

/// Opens the window (with this PDF, if given) and returns when it is closed.
pub fn lancer(fichier: Option<&Path>) -> Resultat<()> {
    let icone = eframe::icon_data::from_png_bytes(include_bytes!("../../../../assets/icone/massicot-256.png")).ok();
    let mut fenetre = egui::ViewportBuilder::default().with_title("Massicot").with_inner_size([1180.0, 760.0]).with_min_inner_size([900.0, 560.0]);
    if let Some(i) = icone {
        fenetre = fenetre.with_icon(std::sync::Arc::new(i));
    }
    let options = eframe::NativeOptions { viewport: fenetre, ..Default::default() };
    let fichier = fichier.map(Path::to_path_buf);
    eframe::run_native("Massicot", options, Box::new(move |cc| Ok(Box::new(Atelier::new(&cc.egui_ctx, fichier.as_deref())))))
        .map_err(|e| Erreur::new(format!("la fenêtre n'a pas pu s'ouvrir (détail : {e})")))
}

/// Tells about a window that could not open: started from the Start menu, there is no console
/// to read the message in.
pub fn signaler(message: &str) {
    #[cfg(windows)]
    {
        rfd::MessageDialog::new()
            .set_title("Massicot")
            .set_description(format!("La fenêtre de Massicot n'a pas pu s'ouvrir.\n\n{message}"))
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
    #[cfg(not(windows))]
    let _ = message;
}

/// The open PDF.
struct Document {
    chemin: PathBuf,
    nom: String,
    pages: usize,
    infos: Value,
}

/// What a running job is for.
enum Pour {
    Outil(String),
    Decoupe,
}

/// A tool running in the background.
struct Tache {
    pour: Pour,
    recu: Receiver<Result<(Value, String), String>>,
    debut: Instant,
}

/// The last result of a tool.
struct Issue {
    reussi: bool,
    texte: String,
    dossier: Option<PathBuf>,
}

/// The window's state.
pub struct Atelier {
    outils: Vec<Outil>,
    choisi: Option<String>,
    doc: Option<Document>,
    vignettes: Option<Vignettes>,
    selection: BTreeSet<usize>,
    dernier_clic: Option<usize>,
    formulaires: BTreeMap<String, Formulaire>,
    issues: BTreeMap<String, Issue>,
    decoupe: Decoupe,
    tache: Option<Tache>,
    etat: String,
    /// Durations in the status bar (off in the screen tests: they vary from run to run).
    durees: bool,
}

impl Atelier {
    pub fn new(ctx: &egui::Context, fichier: Option<&Path>) -> Atelier {
        theme::appliquer(ctx);
        let mut a = Atelier {
            outils: outils::outils(),
            choisi: None,
            doc: None,
            vignettes: None,
            selection: BTreeSet::new(),
            dernier_clic: None,
            formulaires: BTreeMap::new(),
            issues: BTreeMap::new(),
            decoupe: Decoupe::default(),
            tache: None,
            etat: format!("Prêt · Massicot {}", crate::VERSION),
            durees: true,
        };
        if let Some(f) = fichier {
            a.ouvrir(ctx, f);
        }
        a
    }

    /// The tool shown on the right (`None`: the PDF's information).
    pub fn choisir_outil(&mut self, outil: Option<&str>) {
        self.choisi = outil.filter(|o| *o != "infos").map(str::to_string);
        if let Some(o) = &self.choisi
            && !self.formulaires.contains_key(o)
            && let Some(def) = self.outils.iter().find(|t| &t.nom == o)
        {
            let mut f = Formulaire::nouveau(def, self.doc.as_ref().map(|d| d.chemin.as_path()));
            f.suivre_selection(&self.selection);
            self.formulaires.insert(o.clone(), f);
        }
    }

    /// Opens a PDF: its information (the `infos` tool), then its thumbnails.
    pub fn ouvrir(&mut self, ctx: &egui::Context, chemin: &Path) {
        let mut args = Map::new();
        args.insert("fichier".into(), json!(chemin.to_string_lossy()));
        match outils::executer_outil("infos", &args, &|t| Ok(t.to_string())) {
            Ok((infos, _)) => {
                let pages = infos["pages"].as_u64().unwrap_or(0) as usize;
                let nom = chemin.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.etat = format!("{nom} ouvert · {pages} pages");
                self.vignettes = Some(Vignettes::demarrer(ctx, chemin, pages));
                self.doc = Some(Document { chemin: chemin.to_path_buf(), nom, pages, infos });
                self.selection.clear();
                self.dernier_clic = None;
                self.issues.clear();
                for f in self.formulaires.values_mut() {
                    f.suivre_document(Some(chemin));
                    f.pages_auto = true;
                    f.suivre_selection(&self.selection);
                }
                self.decoupe.suivre_document(Some(chemin));
            }
            Err(e) => self.etat = format!("Impossible d'ouvrir ce PDF : {e}"),
        }
    }

    /// The thumbnails are all drawn (for the screen tests).
    pub fn vignettes_pretes(&self) -> bool {
        self.vignettes.as_ref().is_none_or(Vignettes::toutes_pretes)
    }

    /// No tool is running (for the screen tests).
    pub fn libre(&self) -> bool {
        self.tache.is_none()
    }

    /// Selects these pages (0-based), as clicks on the thumbnails do (for the screen tests).
    pub fn selectionner_pour_essai(&mut self, pages: &[usize]) {
        self.selection = pages.iter().copied().collect();
        for f in self.formulaires.values_mut() {
            f.suivre_selection(&self.selection);
        }
    }

    /// Without durations in the status bar (for the screen tests).
    pub fn sans_durees(&mut self) {
        self.durees = false;
    }

    /// The cut at each header (for the screen tests).
    pub fn decoupe_mut(&mut self) -> &mut Decoupe {
        &mut self.decoupe
    }

    fn lancer_tache(&mut self, ctx: &egui::Context, pour: Pour, outil: &str, args: Map<String, Value>) {
        let (envoi, recu) = channel();
        let (ctx2, outil2) = (ctx.clone(), outil.to_string());
        std::thread::spawn(move || {
            let r = outils::executer_outil(&outil2, &args, &|t| Ok(t.to_string()));
            let _ = envoi.send(r);
            ctx2.request_repaint();
        });
        self.tache = Some(Tache { pour, recu, debut: Instant::now() });
    }

    fn suivre_tache(&mut self, ctx: &egui::Context) {
        let Some(t) = &self.tache else { return };
        let Ok(r) = t.recu.try_recv() else {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
            return;
        };
        let duree = if self.durees { format!(" en {} s", t.debut.elapsed().as_secs()) } else { String::new() };
        let pour = std::mem::replace(&mut self.tache, None).map(|t| t.pour);
        match pour {
            Some(Pour::Decoupe) => {
                self.etat = match &r {
                    Ok(_) => format!("Terminé{duree}"),
                    Err(_) => "La découpe n'a pas pu se faire".into(),
                };
                self.decoupe.recevoir(r.map(|(json, _)| json));
            }
            Some(Pour::Outil(o)) => {
                let dossier = self.formulaires.get(&o).and_then(Formulaire::dossier_du_resultat);
                let issue = match r {
                    Ok((_, texte)) => Issue { reussi: true, texte, dossier },
                    Err(e) => Issue { reussi: false, texte: e, dossier: None },
                };
                self.etat = if issue.reussi { format!("Terminé{duree}") } else { "Échec : voir le message à droite".into() };
                self.issues.insert(o, issue);
            }
            None => {}
        }
    }

    fn colonne_outils(&mut self, ui: &mut egui::Ui) {
        let mut clic = None;
        for (groupe, outils) in GROUPES {
            ui.add_space(6.0);
            ui.label(RichText::new(groupe.to_uppercase()).size(11.0).family(theme::gras()).color(GRIS));
            for (commande, nom, symbole) in outils {
                let choisi = self.choisi.as_deref() == Some(*commande);
                let (fond, encre, pastille, symbole_encre) = if choisi {
                    (BLEU, Color32::WHITE, Color32::from_white_alpha(56), Color32::WHITE)
                } else {
                    (Color32::TRANSPARENT, TEXTE, theme::BLEU_PALE, theme::BLEU_FONCE)
                };
                let (rect, reponse) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), Sense::click());
                let fond = if !choisi && reponse.hovered() { Color32::from_rgb(0xEC, 0xEE, 0xF2) } else { fond };
                let p = ui.painter();
                p.rect_filled(rect, CornerRadius::same(7), fond);
                let case = egui::Rect::from_center_size(egui::pos2(rect.left() + 18.0, rect.center().y), egui::vec2(20.0, 20.0));
                p.rect_filled(case, CornerRadius::same(5), pastille);
                let corps = if symbole.chars().count() > 1 { 10.0 } else { 12.0 };
                p.text(case.center(), egui::Align2::CENTER_CENTER, *symbole, egui::FontId::new(corps, theme::gras()), symbole_encre);
                p.text(egui::pos2(rect.left() + 36.0, rect.center().y), egui::Align2::LEFT_CENTER, *nom, egui::FontId::proportional(13.5), encre);
                reponse.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, choisi, *nom));
                if reponse.clicked() {
                    clic = Some(*commande);
                }
            }
        }
        if let Some(c) = clic {
            self.choisir_outil(Some(c));
        }
    }

    fn barre_document(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new().fill(Color32::WHITE).inner_margin(egui::Margin::symmetric(12, 7)).stroke(Stroke::new(1.0, BORDURE)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let ouvrir = ui.button("Ouvrir un PDF…");
                if ouvrir.clicked()
                    && let Some(f) = dialogues::un_pdf()
                {
                    self.ouvrir(ui.ctx(), Path::new(&f));
                }
                if let Some(d) = &self.doc {
                    ui.label(theme::texte_gras(d.nom.as_str(), 13.5));
                    let n = d.pages;
                    let sel = self.selection.len();
                    let mut info = if n > 1 { format!("{n} pages") } else { format!("{n} page") };
                    if sel > 0 {
                        info.push_str(&format!(" · {sel} sélectionnée{}", if sel > 1 { "s" } else { "" }));
                    }
                    ui.label(RichText::new(info).color(GRIS));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Glissez un PDF ici pour l'ouvrir").color(GRIS));
                });
            });
        });
    }

    /// One thumbnail; returns the click response. `bande`: the top band searched for the header (%).
    fn vignette(&self, ui: &mut egui::Ui, page: usize, largeur: f32, bande: Option<u8>) -> egui::Response {
        let texture = self.vignettes.as_ref().and_then(|v| v.page(page));
        let ratio = texture.map(|t| t.size_vec2().y / t.size_vec2().x.max(1.0)).unwrap_or(1.414);
        let taille_numero = if largeur < 80.0 { 10.0 } else { 11.5 };
        let (rect, reponse) = ui.allocate_exact_size(egui::vec2(largeur, largeur * ratio + taille_numero + 6.0), Sense::click());
        let feuille = egui::Rect::from_min_size(rect.min, egui::vec2(largeur, largeur * ratio));
        let p = ui.painter();
        p.rect_filled(feuille.translate(egui::vec2(0.0, 1.0)).expand(1.0), CornerRadius::ZERO, Color32::from_black_alpha(36));
        p.rect_filled(feuille, CornerRadius::ZERO, Color32::WHITE);
        if let Some(t) = texture {
            p.image(t.id(), feuille, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        }
        if let Some(pourcent) = bande {
            let h = feuille.height() * f32::from(pourcent) / 100.0;
            let zone = egui::Rect::from_min_size(feuille.min, egui::vec2(feuille.width(), h));
            p.rect_filled(zone, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(242, 183, 5, 56));
            p.hline(zone.x_range(), zone.bottom(), Stroke::new(1.5, Color32::from_rgb(0xD9, 0xA3, 0x00)));
        }
        if self.selection.contains(&page) {
            p.rect_stroke(feuille, CornerRadius::ZERO, Stroke::new(2.5, BLEU), StrokeKind::Outside);
        }
        p.text(egui::pos2(rect.center().x, feuille.bottom() + 4.0), egui::Align2::CENTER_TOP, format!("{}", page + 1), egui::FontId::proportional(taille_numero), GRIS);
        reponse
    }

    fn selectionner(&mut self, page: usize, modificateurs: egui::Modifiers) {
        if modificateurs.shift
            && let Some(d) = self.dernier_clic
        {
            for p in d.min(page)..=d.max(page) {
                self.selection.insert(p);
            }
        } else if modificateurs.command {
            if !self.selection.remove(&page) {
                self.selection.insert(page);
            }
        } else if self.selection.len() == 1 && self.selection.contains(&page) {
            self.selection.clear();
        } else {
            self.selection = BTreeSet::from([page]);
        }
        self.dernier_clic = Some(page);
        for f in self.formulaires.values_mut() {
            f.suivre_selection(&self.selection);
        }
    }

    fn pages(&mut self, ui: &mut egui::Ui) {
        let Some(doc) = &self.doc else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Ouvrez un PDF (« Ouvrir un PDF… ») ou glissez-le ici.").size(15.0).color(GRIS));
            });
            return;
        };
        let n = doc.pages;
        let en_decoupe = self.choisi.as_deref() == Some("decouper-regex");
        let blocs = if en_decoupe { self.decoupe.blocs() } else { Vec::new() };
        let bande = (en_decoupe && self.decoupe.etape == 2).then_some(self.decoupe.zone_haute);
        let mut clics = Vec::new();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::new().inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                if blocs.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(16.0, 14.0);
                        for page in 0..n {
                            let r = self.vignette(ui, page, LARGEUR_VIGNETTE, bande);
                            if r.clicked() {
                                clics.push((page, ui.input(|i| i.modifiers)));
                            }
                        }
                    });
                } else {
                    // Blocks flow in rows, never split: each one keeps its pages side by side.
                    let largeur_bloc = |debut: usize, fin: usize| ((fin - debut + 1) as f32 * 68.0 - 6.0).max(118.0) + 19.0;
                    let mut rangees: Vec<Vec<&(String, usize, usize, bool)>> = vec![Vec::new()];
                    let mut x = 0.0;
                    for b in &blocs {
                        let l = largeur_bloc(b.1, b.2);
                        if x > 0.0 && x + l > ui.available_width() {
                            rangees.push(Vec::new());
                            x = 0.0;
                        }
                        rangees.last_mut().map(|r| r.push(b));
                        x += l + 12.0;
                    }
                    for rangee in rangees {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                            for (fichier, debut, fin, alerte) in rangee {
                                let (fond, bord, encre) = if *alerte {
                                    (Color32::from_rgba_unmultiplied(242, 183, 5, 30), Color32::from_rgb(0xE3, 0xA9, 0x00), theme::JAUNE_FONCE)
                                } else {
                                    (Color32::from_rgba_unmultiplied(47, 111, 214, 20), Color32::from_rgba_unmultiplied(47, 111, 214, 90), theme::BLEU_FONCE)
                                };
                                egui::Frame::new().fill(fond).stroke(Stroke::new(1.5, bord)).corner_radius(9).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
                                    ui.set_min_width(largeur_bloc(*debut, *fin) - 19.0);
                                    ui.vertical(|ui| {
                                        let pages = if debut == fin { format!("p. {debut}") } else { format!("p. {debut}-{fin}") };
                                        let marque = if *alerte { "⚠ " } else { "" };
                                        let nom = fichier.trim_end_matches(".pdf");
                                        ui.label(RichText::new(format!("{marque}{nom} · {pages}")).size(11.5).family(theme::gras()).color(encre));
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 6.0;
                                            for page in (debut - 1)..*fin {
                                                let r = self.vignette(ui, page, 62.0, None);
                                                if r.clicked() {
                                                    clics.push((page, ui.input(|i| i.modifiers)));
                                                }
                                            }
                                        });
                                    });
                                });
                            }
                        });
                        ui.add_space(12.0);
                    }
                }
            });
        });
        for (page, m) in clics {
            self.selectionner(page, m);
        }
    }

    fn panneau_infos(&self, ui: &mut egui::Ui) {
        let Some(doc) = &self.doc else {
            ui.heading("Bienvenue");
            ui.label("Ouvrez un PDF, puis choisissez un outil à gauche.");
            ui.label(RichText::new("Les pages sélectionnées au centre servent aux outils qui travaillent sur des pages (extraire, tourner, supprimer…).").color(GRIS));
            return;
        };
        ui.heading(doc.nom.as_str());
        let i = &doc.infos;
        theme::carte(Color32::WHITE, BORDURE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let formats: Vec<String> = i["formats"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|f| f["format"].as_str().map(str::to_string).unwrap_or_else(|| format!("{:.0} × {:.0} mm", f["largeur_mm"].as_f64().unwrap_or(0.0), f["hauteur_mm"].as_f64().unwrap_or(0.0))))
                .collect();
            ui.label(format!("{} pages · {}", doc.pages, formats.join(", ")));
            let taille = i["taille_octets"].as_f64().unwrap_or(0.0) / 1_048_576.0;
            ui.label(format!("Taille : {taille:.1} Mo · PDF {}", i["version_pdf"].as_str().unwrap_or("?")).replace('.', ","));
            let protection = if i["protege"].as_bool() == Some(true) { "oui" } else { "aucune" };
            ui.label(format!("Signets : {} · Protection : {protection}", i["signets"].as_u64().unwrap_or(0)));
        });
        ui.label(RichText::new("Choisissez un outil à gauche. Les pages sélectionnées au centre servent aux outils qui travaillent sur des pages (extraire, tourner, supprimer…).").color(GRIS));
    }

    fn panneau_outil(&mut self, ui: &mut egui::Ui, outil: &str) {
        let occupe = self.tache.is_some();
        let doc = self.doc.as_ref().map(|d| d.chemin.clone());
        if outil == "decouper-regex" {
            match self.decoupe.afficher(ui, doc.as_deref(), occupe) {
                Demande::Lancer(args) => {
                    self.etat = "Lecture des pages…".into();
                    self.lancer_tache(ui.ctx(), Pour::Decoupe, "decouper-regex", args);
                }
                Demande::OuvrirDossier(d) => dialogues::ouvrir_dossier(&d),
                Demande::Rien => {}
            }
            return;
        }
        let Some(nom) = GROUPES.iter().flat_map(|(_, l)| l.iter()).find(|(c, ..)| *c == outil).map(|(_, n, _)| *n) else { return };
        ui.heading(nom);
        let mut lancer = None;
        if let Some(f) = self.formulaires.get_mut(outil) {
            ui.label(RichText::new(f.outil.description.as_str()).color(GRIS));
            ui.add_space(4.0);
            egui::ScrollArea::vertical().max_height((ui.available_height() - 140.0).max(120.0)).auto_shrink([false, true]).show(ui, |ui| {
                if f.afficher(ui) {
                    self.issues.remove(outil);
                }
            });
            ui.add_space(6.0);
            let texte = if occupe { "En cours…" } else { formulaire::verbe(outil) };
            if ui.add_enabled(!occupe, theme::bouton_plein(texte, BLEU, Color32::WHITE).min_size(egui::vec2(ui.available_width(), 32.0))).clicked() {
                lancer = Some(f.arguments());
            }
        }
        if let Some(args) = lancer {
            self.etat = format!("{nom} en cours…");
            self.lancer_tache(ui.ctx(), Pour::Outil(outil.to_string()), outil, args);
        }
        if let Some(issue) = self.issues.get(outil) {
            ui.add_space(4.0);
            if issue.reussi {
                bandeau(ui, true, "✓ Fait");
                egui::ScrollArea::vertical().id_salt("resultat").max_height(ui.available_height() - 36.0).show(ui, |ui| {
                    ui.label(RichText::new(issue.texte.trim_end()).monospace().size(11.5));
                });
                if let Some(d) = &issue.dossier
                    && ui.button("Ouvrir le dossier").clicked()
                {
                    dialogues::ouvrir_dossier(d);
                }
            } else {
                bandeau(ui, false, &issue.texte);
            }
        }
    }
}

impl eframe::App for Atelier {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Some(v) = &mut self.vignettes {
            v.recevoir(&ctx);
        }
        self.suivre_tache(&ctx);
        let deposes: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        if let Some(pdf) = deposes.iter().find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"))) {
            self.ouvrir(&ctx, pdf);
        }

        egui::Panel::bottom("etat").exact_size(28.0).frame(theme::cadre(FOND_ETAT, 0).inner_margin(egui::Margin::symmetric(12, 5))).show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(t) = &self.tache {
                    ui.spinner();
                    let duree = if self.durees { format!(" · {} s", t.debut.elapsed().as_secs()) } else { String::new() };
                    ui.label(RichText::new(format!("{}{duree}", self.etat)).small().color(GRIS));
                } else {
                    ui.label(RichText::new(self.etat.as_str()).small().color(GRIS));
                }
                if let Some(e) = self.vignettes.as_ref().and_then(|v| v.erreur.as_ref()) {
                    ui.label(RichText::new(format!("Aperçu incomplet : {e}")).small().color(theme::ROUGE));
                }
            });
        });
        egui::Panel::left("outils").exact_size(214.0).resizable(false).frame(theme::cadre(FOND_OUTILS, 10)).show(ui, |ui| self.colonne_outils(ui));
        egui::Panel::right("reglages").exact_size(330.0).resizable(false).frame(theme::cadre(Color32::WHITE, 16)).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            match self.choisi.clone() {
                Some(o) => self.panneau_outil(ui, &o),
                None => self.panneau_infos(ui),
            }
        });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(FOND_PAGES)).show(ui, |ui| {
            self.barre_document(ui);
            self.pages(ui);
        });
    }
}
