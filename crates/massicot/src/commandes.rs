//! The commands: their arguments (French help) and how each one runs. The command line and the
//! MCP server both go through [`executer`].

use std::path::PathBuf;

use clap::{ArgGroup, Args, Subcommand, ValueHint};
use serde::Serialize;

use crate::erreur::{Erreur, Resultat};
use crate::lecture::{ModeOcr, MoteurOcr};
use crate::ops;
use crate::ops::pages::{Position, Selection};

/// Help layout of a command (clap's own headings are English).
pub const CMD_TEMPLATE: &str = "\
{about}

Utilisation : {usage}

Arguments :
{positionals}

Options :
{options}{after-help}";

#[derive(Debug, Subcommand)]
pub enum Commande {
    /// Décrit un PDF : pages, formats, métadonnées, signets, protection.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot infos <FICHIER> [--json]")]
    Infos(InfosArgs),

    /// Fusionne plusieurs PDF en un seul, dans l'ordre donné.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot fusionner <FICHIERS>... --sortie <FICHIER> [options]")]
    Fusionner(FusionnerArgs),

    /// Extrait des pages dans un nouveau PDF, dans l'ordre donné.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot extraire <FICHIER> --pages <PAGES> --sortie <FICHIER> [options]")]
    Extraire(ExtraireArgs),

    /// Découpe un PDF en plusieurs : toutes les N pages, avant des pages, aux signets d'un niveau, ou par taille.
    #[command(
        help_template = CMD_TEMPLATE,
        override_usage = "massicot decouper <FICHIER> (--toutes <N> | --avant <PAGES> | --signets <NIVEAU> | --taille <TAILLE>) --dossier <DOSSIER> [options]"
    )]
    Decouper(DecouperArgs),

    /// Tourne des pages : toutes, une sélection, les paires ou les impaires.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot tourner <FICHIER> --angle <ANGLE> [--pages <PAGES> | --paires | --impaires] --sortie <FICHIER> [options]")]
    Tourner(TournerArgs),

    /// Supprime des pages.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot supprimer <FICHIER> (--pages <PAGES> | --paires | --impaires) --sortie <FICHIER> [options]")]
    Supprimer(SupprimerArgs),

    /// Déplace des pages à une autre position.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot deplacer <FICHIER> --pages <PAGES> --vers <POSITION> --sortie <FICHIER> [options]")]
    Deplacer(DeplacerArgs),

    /// Inverse l'ordre des pages (toutes, ou une sélection).
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot inverser <FICHIER> [--pages <PAGES> | --paires | --impaires] --sortie <FICHIER> [options]")]
    Inverser(InverserArgs),

    /// Donne le texte de chaque page : la couche texte du PDF, ou l'OCR pour les pages scannées.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot texte <FICHIER> [--pages <PAGES>] [--ocr auto|toujours|jamais] [options]")]
    Texte(TexteArgs),

    /// Reconnaît le texte des pages scannées (OCR) et l'ajoute en texte invisible : le PDF devient cherchable.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot ocr <FICHIER> --sortie <FICHIER> [options]")]
    Ocr(OcrArgs),

    /// Découpe un lot (souvent scanné) à chaque page qui porte un en-tête reconnu par une expression régulière : un PDF par bloc, sans IA. Sans --couper, mode à blanc : rien n'est écrit, un rapport dit quelles pages ouvrent quel bloc.
    #[command(
        name = "decouper-regex",
        help_template = CMD_TEMPLATE,
        override_usage = "massicot decouper-regex <FICHIER> --regex <REGEX>... --dossier <DOSSIER> [--modele <MODELE>] [--zone-haute <POURCENT>] [--couper <JETON>] [options]",
        after_help = "\nExemple : massicot decouper-regex lot.pdf --regex \"PROCÈS-VERBAL N° (?<num>\\d+)\" --zone-haute 30 --modele \"PV-{num}.pdf\" --dossier PV\nPuis, après lecture du rapport : la même commande avec --couper <JETON> (jeton donné par le mode à blanc)."
    )]
    DecouperRegex(DecouperRegexArgs),

    /// Lance le serveur MCP (stdio) : Claude pilote alors toutes les commandes, dans le dossier racine.
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot mcp --racine <DOSSIER>")]
    Mcp(McpArgs),

    /// Mélange deux PDF page à page (recto et verso scannés à part) : 1-A, 2-B, 3-C…
    #[command(help_template = CMD_TEMPLATE, override_usage = "massicot melanger <RECTO> <VERSO> --sortie <FICHIER> [--verso-inverse] [options]")]
    Melanger(MelangerArgs),

    /// Accueil de la console ouverte par le raccourci du menu Démarrer.
    #[command(hide = true, help_template = CMD_TEMPLATE, override_usage = "massicot accueil")]
    Accueil,
}

/// The name each command has on the command line (and as an MCP tool).
pub fn nom(commande: &Commande) -> &'static str {
    match commande {
        Commande::Infos(_) => "infos",
        Commande::Fusionner(_) => "fusionner",
        Commande::Extraire(_) => "extraire",
        Commande::Decouper(_) => "decouper",
        Commande::Tourner(_) => "tourner",
        Commande::Supprimer(_) => "supprimer",
        Commande::Deplacer(_) => "deplacer",
        Commande::Inverser(_) => "inverser",
        Commande::Melanger(_) => "melanger",
        Commande::Texte(_) => "texte",
        Commande::Mcp(_) => "mcp",
        Commande::DecouperRegex(_) => "decouper-regex",
        Commande::Ocr(_) => "ocr",
        Commande::Accueil => "accueil",
    }
}

const AIDE_PAGES: &str = "Pages, comptées à partir de 1 : « 1-3,5,8- » (« 8- » : jusqu'à la fin ; « fin » : la dernière page)";

#[derive(Debug, Args)]
pub struct InfosArgs {
    /// Le PDF à décrire.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,
}

#[derive(Debug, Args)]
pub struct Ecriture {
    /// Le PDF à écrire.
    #[arg(short = 'o', long, value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub sortie: PathBuf,

    /// Remplace le fichier de sortie s'il existe déjà.
    #[arg(long)]
    pub ecraser: bool,
}

#[derive(Debug, Args)]
pub struct FusionnerArgs {
    /// Les PDF à fusionner, dans l'ordre (au moins deux).
    #[arg(value_name = "FICHIERS", required = true, num_args = 2.., value_hint = ValueHint::FilePath)]
    pub fichiers: Vec<PathBuf>,

    /// N'ajoute aucun signet (par défaut : un signet par fichier, qui garde ses propres signets).
    #[arg(long)]
    pub sans_signets: bool,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

#[derive(Debug, Args)]
pub struct ExtraireArgs {
    /// Le PDF source.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    #[arg(long, value_name = "PAGES", help = AIDE_PAGES)]
    pub pages: String,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("mode").required(true).args(["toutes", "avant", "signets", "taille"])))]
pub struct DecouperArgs {
    /// Le PDF à découper.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    /// Un fichier toutes les N pages.
    #[arg(long, value_name = "N", value_parser = entier_positif)]
    pub toutes: Option<usize>,

    /// Un nouveau fichier commence à chacune de ces pages (ex. « 4,9 »).
    #[arg(long, value_name = "PAGES")]
    pub avant: Option<String>,

    /// Un fichier par signet de ce niveau (1 : signets principaux), nommé d'après le signet.
    #[arg(long, value_name = "NIVEAU", value_parser = entier_positif)]
    pub signets: Option<usize>,

    /// Fichiers d'au plus cette taille (ex. « 5Mo », « 500ko »).
    #[arg(long, value_name = "TAILLE", value_parser = ops::decoupe::lire_taille)]
    pub taille: Option<u64>,

    /// Le dossier où écrire les fichiers (créé au besoin).
    #[arg(short = 'd', long, value_name = "DOSSIER", value_hint = ValueHint::DirPath)]
    pub dossier: PathBuf,

    /// Début du nom des fichiers (par défaut : le nom du PDF découpé).
    #[arg(long, value_name = "NOM")]
    pub prefixe: Option<String>,

    /// Remplace les fichiers qui existent déjà.
    #[arg(long)]
    pub ecraser: bool,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("selection").args(["pages", "paires", "impaires"])))]
pub struct SelectionArgs {
    #[arg(long, value_name = "PAGES", help = AIDE_PAGES)]
    pub pages: Option<String>,

    /// Les pages paires (2, 4, 6…).
    #[arg(long)]
    pub paires: bool,

    /// Les pages impaires (1, 3, 5…).
    #[arg(long)]
    pub impaires: bool,
}

impl SelectionArgs {
    fn selection(&self) -> Selection {
        match (&self.pages, self.paires, self.impaires) {
            (Some(p), _, _) => Selection::Pages(p.clone()),
            (None, true, _) => Selection::Paires,
            (None, false, true) => Selection::Impaires,
            _ => Selection::Toutes,
        }
    }

    fn est_vide(&self) -> bool {
        self.selection() == Selection::Toutes
    }
}

#[derive(Debug, Args)]
pub struct TournerArgs {
    /// Le PDF source.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    /// Angle dans le sens des aiguilles d'une montre : 90, 180, 270, ou -90 vers la gauche.
    #[arg(long, value_name = "ANGLE", allow_negative_numbers = true, value_parser = angle)]
    pub angle: i64,

    #[command(flatten)]
    pub selection: SelectionArgs,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

#[derive(Debug, Args)]
pub struct SupprimerArgs {
    /// Le PDF source.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    #[command(flatten)]
    pub selection: SelectionArgs,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

#[derive(Debug, Args)]
pub struct DeplacerArgs {
    /// Le PDF source.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    #[arg(long, value_name = "PAGES", help = AIDE_PAGES)]
    pub pages: String,

    /// Où les placer : « debut », « fin », ou la position (à partir de 1) de la première dans le résultat.
    #[arg(long, value_name = "POSITION", value_parser = position)]
    pub vers: Position,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

#[derive(Debug, Args)]
pub struct InverserArgs {
    /// Le PDF source.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    #[command(flatten)]
    pub selection: SelectionArgs,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

const AIDE_MOTEUR: &str = "Moteur d'OCR : « auto » (Tesseract en français s'il est installé, sinon le moteur intégré), « integre » (sans installation, mais il lit les lettres accentuées comme « ? »), « tesseract » (installé à part, langue française)";

#[derive(Debug, Args)]
pub struct TexteArgs {
    /// Le PDF à lire.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    #[arg(long, value_name = "PAGES", help = AIDE_PAGES)]
    pub pages: Option<String>,

    /// Quand passer par l'OCR : « auto » (pages sans couche texte, par défaut), « toujours », « jamais ».
    #[arg(long, value_name = "QUAND", value_enum, default_value_t = ModeOcr::Auto, hide_default_value = true, hide_possible_values = true)]
    pub ocr: ModeOcr,

    #[arg(long, value_name = "MOTEUR", value_enum, default_value_t = MoteurOcr::Auto, hide_default_value = true, hide_possible_values = true, help = AIDE_MOTEUR)]
    pub moteur: MoteurOcr,
}

#[derive(Debug, Args)]
pub struct OcrArgs {
    /// Le PDF scanné.
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    /// Reconnaît aussi les pages qui ont déjà du texte (par défaut, elles sont laissées telles quelles).
    #[arg(long)]
    pub toutes_les_pages: bool,

    #[arg(long, value_name = "MOTEUR", value_enum, default_value_t = MoteurOcr::Auto, hide_default_value = true, hide_possible_values = true, help = AIDE_MOTEUR)]
    pub moteur: MoteurOcr,

    /// Résolution de lecture des pages, en points par pouce (300 par défaut).
    #[arg(long, value_name = "DPI", default_value_t = 300, hide_default_value = true, value_parser = dpi)]
    pub dpi: u32,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

#[derive(Debug, Args)]
pub struct DecouperRegexArgs {
    /// Le PDF à découper (lot scanné ou PDF texte).
    #[arg(value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub fichier: PathBuf,

    /// Expression régulière de l'en-tête, par ex. « PROCÈS-VERBAL N° (?<num>\d+) ». Plusieurs --regex possibles : une page qui en vérifie une ouvre un bloc.
    #[arg(long = "regex", value_name = "REGEX", required = true)]
    pub regex: Vec<String>,

    /// Le dossier où écrire les fichiers et le rapport (créé au besoin).
    #[arg(short = 'd', long, value_name = "DOSSIER", value_hint = ValueHint::DirPath)]
    pub dossier: PathBuf,

    /// Nom des fichiers : {bloc} (numéro du bloc), {page}, {fichier} (nom du lot), {texte} (en-tête trouvé) et les groupes de la regex ({1}, {num}…). Par défaut « {fichier}-{bloc}.pdf ».
    #[arg(long, value_name = "MODELE", default_value = ops::entetes::MODELE_PAR_DEFAUT, hide_default_value = true)]
    pub modele: String,

    /// Ne cherche l'en-tête que dans le haut de chaque page : ce pourcentage de sa hauteur (par ex. 30).
    #[arg(long, value_name = "POURCENT", value_parser = pourcent)]
    pub zone_haute: Option<u8>,

    /// Quand passer par l'OCR : « auto » (pages sans couche texte, par défaut), « toujours », « jamais ».
    #[arg(long, value_name = "QUAND", value_enum, default_value_t = ModeOcr::Auto, hide_default_value = true, hide_possible_values = true)]
    pub ocr: ModeOcr,

    #[arg(long, value_name = "MOTEUR", value_enum, default_value_t = MoteurOcr::Auto, hide_default_value = true, hide_possible_values = true, help = AIDE_MOTEUR)]
    pub moteur: MoteurOcr,

    /// Résolution de lecture des pages par OCR, en points par pouce (300 par défaut).
    #[arg(long, value_name = "DPI", default_value_t = 300, hide_default_value = true, value_parser = dpi)]
    pub dpi: u32,

    /// Tient compte des majuscules et minuscules (ignorées par défaut).
    #[arg(long)]
    pub respecter_casse: bool,

    /// Tient compte des accents (ignorés par défaut).
    #[arg(long)]
    pub respecter_accents: bool,

    /// Pages avant le premier en-tête : « a-part » (un fichier à part, par défaut), « premier » (jointes au premier bloc) ou « ignorer ».
    #[arg(long, value_name = "CHOIX", value_enum, default_value_t = ops::entetes::AvantPremier::APart, hide_default_value = true, hide_possible_values = true)]
    pub avant_premier: ops::entetes::AvantPremier,

    /// Coupe pour de bon, avec le jeton donné par le mode à blanc (mêmes fichier et options). Sans elle : mode à blanc.
    #[arg(long, value_name = "JETON")]
    pub couper: Option<String>,

    /// Enregistre aussi le rapport JSON du mode à blanc dans ce fichier.
    #[arg(long, value_name = "FICHIER", value_hint = ValueHint::FilePath)]
    pub rapport: Option<PathBuf>,

    /// Remplace les fichiers qui existent déjà.
    #[arg(long)]
    pub ecraser: bool,
}

#[derive(Debug, Args)]
pub struct McpArgs {
    /// Le dossier où travaille le serveur : les chemins donnés par Claude y sont relatifs et ne peuvent pas en sortir.
    #[arg(long, value_name = "DOSSIER", value_hint = ValueHint::DirPath)]
    pub racine: PathBuf,
}

#[derive(Debug, Args)]
pub struct MelangerArgs {
    /// Le premier PDF (par exemple les rectos).
    #[arg(value_name = "RECTO", value_hint = ValueHint::FilePath)]
    pub premier: PathBuf,

    /// Le second PDF (par exemple les versos).
    #[arg(value_name = "VERSO", value_hint = ValueHint::FilePath)]
    pub second: PathBuf,

    /// Lit le second PDF de la dernière page à la première (versos scannés en ordre inverse).
    #[arg(long)]
    pub verso_inverse: bool,

    /// Pages prises au premier PDF à chaque tour (1 par défaut).
    #[arg(long, value_name = "N", default_value_t = 1, hide_default_value = true, value_parser = entier_positif)]
    pub pas_recto: usize,

    /// Pages prises au second PDF à chaque tour (1 par défaut).
    #[arg(long, value_name = "N", default_value_t = 1, hide_default_value = true, value_parser = entier_positif)]
    pub pas_verso: usize,

    #[command(flatten)]
    pub ecriture: Ecriture,
}

fn entier_positif(s: &str) -> Result<usize, String> {
    s.trim().parse::<usize>().ok().filter(|n| *n > 0).ok_or_else(|| format!("« {s} » n'est pas un nombre entier supérieur à zéro"))
}

fn pourcent(s: &str) -> Result<u8, String> {
    s.trim().trim_end_matches('%').trim().parse::<u8>().ok().filter(|p| (1..=100).contains(p)).ok_or_else(|| "un pourcentage entre 1 et 100".into())
}

fn dpi(s: &str) -> Result<u32, String> {
    s.trim().parse::<u32>().ok().filter(|d| (72..=600).contains(d)).ok_or_else(|| "la résolution doit être un nombre entre 72 et 600".into())
}

fn angle(s: &str) -> Result<i64, String> {
    match s.trim().parse::<i64>() {
        Ok(a) if a % 90 == 0 && a != 0 => Ok(a),
        _ => Err("l'angle doit être 90, 180, 270 ou -90".into()),
    }
}

fn position(s: &str) -> Result<Position, String> {
    match s.trim().to_lowercase().as_str() {
        "debut" | "début" => Ok(Position::Debut),
        "fin" => Ok(Position::Fin),
        n => n.parse::<usize>().ok().filter(|p| *p > 0).map(Position::A).ok_or_else(|| format!("« {s} » : écrivez debut, fin, ou un numéro de position")),
    }
}

/// Runs a parsed command; returns its result as JSON and as French text.
pub fn executer(commande: &Commande) -> Resultat<(serde_json::Value, String)> {
    match commande {
        Commande::Infos(a) => rendu(ops::infos::infos(&a.fichier)?, |r| r.texte()),
        Commande::Fusionner(a) => {
            rendu(ops::fusion::fusionner(&a.fichiers, &a.ecriture.sortie, a.sans_signets, a.ecriture.ecraser)?, |r| r.texte())
        }
        Commande::Extraire(a) => rendu(ops::pages::extraire(&a.fichier, &a.pages, &a.ecriture.sortie, a.ecriture.ecraser)?, |r| r.texte()),
        Commande::Decouper(a) => {
            let mode = match (a.toutes, &a.avant, a.signets, a.taille) {
                (Some(n), ..) => ops::decoupe::Mode::ToutesLes(n),
                (_, Some(p), ..) => ops::decoupe::Mode::Avant(p.clone()),
                (_, _, Some(niveau), _) => ops::decoupe::Mode::Signets(niveau),
                (.., Some(t)) => ops::decoupe::Mode::Taille(t),
                _ => return Err(Erreur::new("choisissez une façon de découper : --toutes, --avant, --signets ou --taille")),
            };
            rendu(ops::decoupe::decouper(&a.fichier, &mode, &a.dossier, a.prefixe.as_deref(), a.ecraser)?, |r| r.texte())
        }
        Commande::Tourner(a) => rendu(
            ops::pages::tourner(&a.fichier, a.angle, &a.selection.selection(), &a.ecriture.sortie, a.ecriture.ecraser)?,
            |r| r.texte(),
        ),
        Commande::Supprimer(a) => {
            if a.selection.est_vide() {
                return Err(Erreur::new("indiquez les pages à supprimer : --pages, --paires ou --impaires"));
            }
            rendu(ops::pages::supprimer(&a.fichier, &a.selection.selection(), &a.ecriture.sortie, a.ecriture.ecraser)?, |r| r.texte())
        }
        Commande::Deplacer(a) => {
            rendu(ops::pages::deplacer(&a.fichier, &a.pages, a.vers, &a.ecriture.sortie, a.ecriture.ecraser)?, |r| r.texte())
        }
        Commande::Inverser(a) => {
            rendu(ops::pages::inverser(&a.fichier, &a.selection.selection(), &a.ecriture.sortie, a.ecriture.ecraser)?, |r| r.texte())
        }
        Commande::Texte(a) => rendu(ops::ocr::texte(&a.fichier, a.pages.as_deref(), a.ocr, a.moteur)?, |r| r.texte()),
        Commande::Ocr(a) => rendu(
            ops::ocr::ocr(&a.fichier, &a.ecriture.sortie, a.toutes_les_pages, a.moteur, a.dpi as f32, a.ecriture.ecraser)?,
            |r| r.texte(),
        ),
        Commande::DecouperRegex(a) => {
            let reglages = ops::entetes::Reglages {
                regex: a.regex.clone(),
                modele: a.modele.clone(),
                ocr: a.ocr,
                moteur: a.moteur,
                dpi: a.dpi,
                zone_haute: a.zone_haute,
                respecter_casse: a.respecter_casse,
                respecter_accents: a.respecter_accents,
                avant_premier: a.avant_premier,
            };
            let rapport = ops::entetes::decouper_regex(&a.fichier, &a.dossier, &reglages, a.couper.as_deref(), a.ecraser)?;
            if let (Some(chemin), true) = (&a.rapport, rapport.a_blanc) {
                ops::entetes::enregistrer_rapport(&rapport, chemin, a.ecraser)?;
            }
            rendu(rapport, |r| r.texte())
        }
        Commande::Mcp(_) => Err(Erreur::new("le serveur MCP se lance depuis la ligne de commande : massicot mcp --racine <DOSSIER>")),
        Commande::Accueil => Ok((serde_json::json!({ "accueil": crate::accueil::texte_simple() }), crate::accueil::texte())),
        Commande::Melanger(a) => {
            let alternance = ops::melange::Alternance { pas_premier: a.pas_recto, pas_second: a.pas_verso, second_inverse: a.verso_inverse };
            rendu(ops::melange::melanger(&a.premier, &a.second, alternance, &a.ecriture.sortie, a.ecriture.ecraser)?, |r| r.texte())
        }
    }
}

fn rendu<T: Serialize>(resultat: T, texte: impl Fn(&T) -> String) -> Resultat<(serde_json::Value, String)> {
    let t = texte(&resultat);
    let v = serde_json::to_value(&resultat).map_err(|e| Erreur::moteur("la mise en forme du résultat", e))?;
    Ok((v, t))
}
