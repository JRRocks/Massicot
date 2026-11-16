//! The commands seen as tools with named parameters, shared by the MCP server and the window:
//! parameters → command line → `clap` → [`executer`]. Both run exactly what the command line
//! runs, with the same checks and the same French messages.

use clap::{ArgAction, CommandFactory, Parser, ValueHint};
use serde_json::{Map, Value};

use crate::cli::Cli;
use crate::commandes::executer;
use crate::erreur::{Erreur, Resultat};

/// Commands that are not tools.
pub const HORS_OUTILS: [&str; 3] = ["mcp", "accueil", "fenetre"];
/// Arguments that are not tool parameters.
pub const HORS_PARAMETRES: [&str; 3] = ["aide", "version", "json"];

/// What kind of value a parameter takes (how the window asks for it).
#[derive(Debug, Clone, PartialEq)]
pub enum Genre {
    /// A yes/no option.
    Case,
    /// One value among these.
    Choix(Vec<String>),
    /// Free text (pages, numbers, expressions…).
    Texte,
    /// Several free values (repeatable option).
    Textes,
    /// An existing PDF to read.
    Fichier,
    /// Several existing PDFs.
    Fichiers,
    /// A PDF (or report) to write.
    Sortie,
    /// A folder.
    Dossier,
}

/// One parameter of a tool.
#[derive(Debug, Clone)]
pub struct Parametre {
    /// Its name (as in MCP and the long option, without `--`).
    pub id: String,
    /// Its French help line.
    pub aide: String,
    pub genre: Genre,
    pub requis: bool,
    /// Its default value, as the help gives it.
    pub defaut: Option<String>,
}

/// A tool: a command and its parameters.
#[derive(Debug, Clone)]
pub struct Outil {
    pub nom: String,
    pub description: String,
    pub parametres: Vec<Parametre>,
}

/// Every tool, in the order of the command line help.
pub fn outils() -> Vec<Outil> {
    Cli::command()
        .get_subcommands()
        .filter(|c| !HORS_OUTILS.contains(&c.get_name()))
        .map(|c| Outil {
            nom: c.get_name().to_string(),
            description: c.get_about().map(|s| s.to_string()).unwrap_or_default(),
            parametres: c
                .get_arguments()
                .filter(|a| !HORS_PARAMETRES.contains(&a.get_id().as_str()))
                .map(|a| {
                    let id = a.get_id().as_str().to_string();
                    let valeurs: Vec<String> = a.get_possible_values().iter().map(|v| v.get_name().to_string()).collect();
                    let plusieurs = matches!(a.get_action(), ArgAction::Append);
                    let genre = match (a.get_action(), a.get_value_hint()) {
                        (ArgAction::SetTrue, _) => Genre::Case,
                        (_, ValueHint::DirPath) => Genre::Dossier,
                        (_, ValueHint::FilePath | ValueHint::AnyPath) if id == "sortie" || id == "rapport" => Genre::Sortie,
                        (_, ValueHint::FilePath | ValueHint::AnyPath) if plusieurs => Genre::Fichiers,
                        (_, ValueHint::FilePath | ValueHint::AnyPath) => Genre::Fichier,
                        _ if !valeurs.is_empty() => Genre::Choix(valeurs),
                        _ if plusieurs => Genre::Textes,
                        _ => Genre::Texte,
                    };
                    let defaut = a.get_default_values().first().map(|v| v.to_string_lossy().into_owned());
                    Parametre { id, aide: a.get_help().map(|h| h.to_string()).unwrap_or_default(), genre, requis: a.is_required_set(), defaut }
                })
                .collect(),
        })
        .collect()
}

fn texte_de(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// The command line equivalent to a tool call; `chemin` turns each path parameter into the path
/// to use (the MCP server keeps them inside its root folder).
pub fn ligne_de_commande(nom: &str, args: &Map<String, Value>, chemin: &dyn Fn(&str) -> Resultat<String>) -> Resultat<Vec<String>> {
    let cli = Cli::command();
    let Some(cmd) = cli.find_subcommand(nom).filter(|c| !HORS_OUTILS.contains(&c.get_name())) else {
        return Err(Erreur::new(format!("outil « {nom} » inconnu")));
    };
    for cle in args.keys() {
        if !cmd.get_arguments().any(|a| a.get_id().as_str() == cle && !HORS_PARAMETRES.contains(&cle.as_str())) {
            return Err(Erreur::new(format!("paramètre « {cle} » inconnu pour l'outil « {nom} »")));
        }
    }
    let mut options = Vec::new();
    let mut positions = Vec::new();
    for a in cmd.get_arguments() {
        let Some(v) = args.get(a.get_id().as_str()) else { continue };
        let est_chemin = matches!(a.get_value_hint(), ValueHint::FilePath | ValueHint::DirPath | ValueHint::AnyPath);
        let valeur = |x: &Value| -> Resultat<String> {
            let t = texte_de(x).ok_or_else(|| Erreur::new(format!("paramètre « {} » : valeur de type inattendu", a.get_id())))?;
            if est_chemin { chemin(&t) } else { Ok(t) }
        };
        let valeurs: Vec<String> = match (a.get_action(), v) {
            (ArgAction::SetTrue, Value::Bool(false)) => continue,
            (ArgAction::SetTrue, Value::Bool(true)) => vec![],
            (ArgAction::SetTrue, _) => return Err(Erreur::new(format!("paramètre « {} » : vrai ou faux attendu", a.get_id()))),
            (_, Value::Array(liste)) => liste.iter().map(valeur).collect::<Resultat<_>>()?,
            (_, x) => vec![valeur(x)?],
        };
        if a.is_positional() {
            positions.push((a.get_index().unwrap_or(0), valeurs));
            continue;
        }
        let long = a.get_long().map(|l| format!("--{l}")).unwrap_or_else(|| format!("--{}", a.get_id()));
        if valeurs.is_empty() {
            options.push(long.clone());
        }
        for v in valeurs {
            options.push(long.clone());
            options.push(v);
        }
    }
    positions.sort_by_key(|(i, _)| *i);
    let mut argv = vec!["massicot".to_string(), nom.to_string()];
    argv.extend(options);
    if !positions.is_empty() {
        argv.push("--".into());
        argv.extend(positions.into_iter().flat_map(|(_, v)| v));
    }
    Ok(argv)
}

/// Runs a tool: Ok((result as JSON, result as French text)) or Err(French message).
pub fn executer_outil(nom: &str, args: &Map<String, Value>, chemin: &dyn Fn(&str) -> Resultat<String>) -> Result<(Value, String), String> {
    let argv = ligne_de_commande(nom, args, chemin).map_err(|e| e.to_string())?;
    let cli = Cli::try_parse_from(&argv).map_err(|e| crate::cli::french_parse_error(&e))?;
    let commande = cli.commande().ok_or_else(|| "commande manquante".to_string())?;
    executer(commande).map_err(|e| e.to_string())
}
