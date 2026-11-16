//! `massicot mcp --racine DIR`: an MCP server on stdio (JSON-RPC 2.0, one message per line).
//! Every command of the command line is a tool, with the same arguments and the same French
//! messages; its schema is built from the command's own definition. File paths stay inside the
//! root folder.

use std::io::{BufRead, Write};
use std::path::{Component, Path, PathBuf};

use clap::{ArgAction, CommandFactory, Parser, ValueHint};
use serde_json::{Map, Value, json};

use crate::cli::Cli;
use crate::commandes::executer;
use crate::erreur::{Erreur, Resultat};

/// Protocol revisions this server speaks (the client's is used when known).
const PROTOCOLES: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
/// Commands that are not tools.
const HORS_OUTILS: [&str; 1] = ["mcp"];
/// Arguments that are not tool parameters.
const HORS_PARAMETRES: [&str; 3] = ["aide", "version", "json"];

const INSTRUCTIONS: &str = "Massicot : boîte à outils PDF hors ligne (fusion, extraction, découpes, rotation, \
suppression, déplacement, inversion, mélange recto-verso, texte des pages, OCR, découpe d'un lot à chaque en-tête \
trouvé par expression régulière). Les chemins sont relatifs au dossier racine du serveur (ou absolus, dans ce \
dossier). Rien n'est écrasé sans « ecraser ». Pour découper un lot de procès-verbaux : appeler « decouper-regex » \
sans « couper » (mode à blanc), lire le rapport (pages de chaque bloc, texte trouvé, alertes), puis rappeler avec \
les mêmes paramètres et « couper » = le jeton du rapport. Le moteur OCR intégré lit les lettres accentuées comme \
« ? » : les regex en tiennent compte d'elles-mêmes pour les pages qu'il lit.";

/// The root folder and the conversion of tool paths into real paths inside it.
pub struct Racine(PathBuf);

/// The real path of an existing file or folder, in the usual Windows form (`C:\…` rather than
/// the `\\?\C:\…` that canonicalisation gives there).
fn reel(p: &Path) -> std::io::Result<PathBuf> {
    let c = p.canonicalize()?;
    Ok(sans_prefixe_verbatim(&c.to_string_lossy()).map(PathBuf::from).unwrap_or(c))
}

fn sans_prefixe_verbatim(s: &str) -> Option<String> {
    if let Some(reste) = s.strip_prefix(r"\\?\UNC\") {
        return Some(format!(r"\\{reste}"));
    }
    let reste = s.strip_prefix(r"\\?\")?;
    (reste.as_bytes().get(1) == Some(&b':')).then(|| reste.to_string())
}

impl Racine {
    pub fn new(dossier: &Path) -> Resultat<Racine> {
        let r = reel(dossier).map_err(|e| Erreur::fichier("ouvrir le dossier racine", dossier, &e))?;
        if !r.is_dir() {
            return Err(Erreur::new(format!("« {} » n'est pas un dossier", dossier.display())));
        }
        Ok(Racine(r))
    }

    pub fn chemin(&self) -> &Path {
        &self.0
    }

    /// A path given to a tool (relative to the root, or absolute inside it), checked.
    pub fn resoudre(&self, donne: &str) -> Resultat<PathBuf> {
        let brut = Path::new(donne.trim());
        let joint = if brut.is_absolute() { brut.to_path_buf() } else { self.0.join(brut) };
        // Lexical clean-up first (no « .. » may climb out), then the real path of what exists.
        let mut propre = PathBuf::new();
        for c in joint.components() {
            match c {
                Component::ParentDir => {
                    propre.pop();
                }
                Component::CurDir => {}
                autre => propre.push(autre),
            }
        }
        let mut existant = propre.as_path();
        let mut reste = Vec::new();
        while !existant.exists() {
            match (existant.parent(), existant.file_name()) {
                (Some(p), Some(n)) => {
                    reste.push(n.to_os_string());
                    existant = p;
                }
                _ => break,
            }
        }
        let mut chemin = reel(existant).unwrap_or_else(|_| existant.to_path_buf());
        for n in reste.into_iter().rev() {
            chemin.push(n);
        }
        if !chemin.starts_with(&self.0) {
            return Err(Erreur::new(format!("chemin « {donne} » refusé : il sort du dossier racine « {} »", self.0.display())));
        }
        Ok(chemin)
    }
}

/// The tools: one per command, with a JSON schema made from its arguments.
pub fn outils() -> Vec<Value> {
    let cli = Cli::command();
    let mut outils: Vec<Value> = cli
        .get_subcommands()
        .filter(|c| !HORS_OUTILS.contains(&c.get_name()))
        .map(|c| {
            let mut proprietes = Map::new();
            let mut requis = Vec::new();
            for a in c.get_arguments().filter(|a| !HORS_PARAMETRES.contains(&a.get_id().as_str())) {
                let id = a.get_id().as_str().to_string();
                let mut aide = a.get_help().map(|h| h.to_string()).unwrap_or_default();
                if matches!(a.get_value_hint(), ValueHint::FilePath | ValueHint::DirPath | ValueHint::AnyPath) {
                    aide.push_str(" (chemin relatif au dossier racine)");
                }
                let mut schema = match a.get_action() {
                    ArgAction::SetTrue => json!({ "type": "boolean" }),
                    ArgAction::Append => json!({ "type": "array", "items": { "type": "string" } }),
                    _ => json!({ "type": "string" }),
                };
                let valeurs: Vec<String> = a.get_possible_values().iter().map(|v| v.get_name().to_string()).collect();
                if !valeurs.is_empty() && !matches!(a.get_action(), ArgAction::SetTrue) {
                    schema["enum"] = json!(valeurs);
                }
                schema["description"] = json!(aide);
                if a.is_required_set() {
                    requis.push(id.clone());
                }
                proprietes.insert(id, schema);
            }
            let mut description = c.get_about().map(|s| s.to_string()).unwrap_or_default();
            for g in c.get_groups().filter(|g| g.is_required_set()) {
                let noms: Vec<String> = g.get_args().map(|a| format!("« {} »", a.as_str())).collect();
                description.push_str(&format!(" Un paramètre parmi : {}.", noms.join(", ")));
            }
            json!({
                "name": c.get_name(),
                "description": description,
                "inputSchema": { "type": "object", "properties": proprietes, "required": requis, "additionalProperties": false },
            })
        })
        .collect();
    outils.push(json!({
        "name": "lister",
        "description": "Liste les PDF du dossier racine (et de ses sous-dossiers), avec leur taille.",
        "inputSchema": {
            "type": "object",
            "properties": { "dossier": { "type": "string", "description": "Sous-dossier à lister (chemin relatif au dossier racine ; par défaut, la racine)" } },
            "additionalProperties": false,
        },
    }));
    outils
}

fn texte_de(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// The command line equivalent to a tool call (paths resolved inside the root).
fn ligne_de_commande(racine: &Racine, nom: &str, args: &Map<String, Value>) -> Resultat<Vec<String>> {
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
        let chemin = matches!(a.get_value_hint(), ValueHint::FilePath | ValueHint::DirPath | ValueHint::AnyPath);
        let valeur = |x: &Value| -> Resultat<String> {
            let t = texte_de(x).ok_or_else(|| Erreur::new(format!("paramètre « {} » : valeur de type inattendu", a.get_id())))?;
            if chemin { Ok(racine.resoudre(&t)?.to_string_lossy().into_owned()) } else { Ok(t) }
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

/// Runs a tool: Ok(result as JSON) or Err(French message).
pub fn appeler(racine: &Racine, nom: &str, args: &Map<String, Value>) -> Result<Value, String> {
    if nom == "lister" {
        return lister(racine, args.get("dossier").and_then(Value::as_str)).map_err(|e| e.to_string());
    }
    let argv = ligne_de_commande(racine, nom, args).map_err(|e| e.to_string())?;
    let cli = Cli::try_parse_from(&argv).map_err(|e| crate::cli::french_parse_error(&e))?;
    let commande = cli.commande().ok_or_else(|| "commande manquante".to_string())?;
    executer(commande).map(|(json, _)| json).map_err(|e| e.to_string())
}

fn lister(racine: &Racine, dossier: Option<&str>) -> Resultat<Value> {
    let depart = racine.resoudre(dossier.unwrap_or("."))?;
    let mut fichiers = Vec::new();
    let mut pile = vec![depart];
    while let Some(d) = pile.pop() {
        let Ok(entrees) = std::fs::read_dir(&d) else { continue };
        for e in entrees.flatten() {
            let p = e.path();
            if p.is_dir() {
                pile.push(p);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("pdf")) && fichiers.len() < 1000 {
                let taille = e.metadata().map(|m| m.len()).unwrap_or(0);
                let relatif = p.strip_prefix(racine.chemin()).unwrap_or(&p).to_string_lossy().replace('\\', "/");
                fichiers.push(json!({ "fichier": relatif, "taille_octets": taille }));
            }
        }
    }
    fichiers.sort_by(|a, b| a["fichier"].as_str().cmp(&b["fichier"].as_str()));
    Ok(json!({ "racine": racine.chemin().display().to_string(), "fichiers": fichiers }))
}

/// Answers one JSON-RPC message; `None` for notifications.
pub fn repondre(racine: &Racine, message: &Value) -> Option<Value> {
    let id = message.get("id").cloned();
    let methode = message.get("method").and_then(Value::as_str).unwrap_or("");
    let id = id?;
    let resultat = match methode {
        "initialize" => {
            let demande = message.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or("");
            let version = PROTOCOLES.iter().find(|p| **p == demande).copied().unwrap_or(PROTOCOLES[1]);
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "massicot", "title": "Massicot", "version": crate::VERSION },
                "instructions": INSTRUCTIONS,
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": outils() }),
        "tools/call" => {
            let nom = message.pointer("/params/name").and_then(Value::as_str).unwrap_or("");
            let vide = Map::new();
            let args = message.pointer("/params/arguments").and_then(Value::as_object).unwrap_or(&vide);
            match appeler(racine, nom, args) {
                Ok(v) => json!({
                    "content": [{ "type": "text", "text": serde_json::to_string(&v).unwrap_or_default() }],
                    "structuredContent": v,
                    "isError": false,
                }),
                Err(e) => json!({ "content": [{ "type": "text", "text": format!("Erreur : {e}") }], "isError": true }),
            }
        }
        autre => {
            return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("méthode inconnue : {autre}") } }));
        }
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": resultat }))
}

/// The server loop on stdin/stdout. Diagnostics go to stderr; stdout carries only messages.
pub fn servir(racine: &Racine) -> Resultat<()> {
    eprintln!("massicot {} : serveur MCP prêt, dossier racine « {} »", crate::VERSION, racine.chemin().display());
    let entree = std::io::stdin();
    let mut sortie = std::io::stdout();
    for ligne in entree.lock().lines() {
        let ligne = ligne.map_err(|e| Erreur::new(format!("lecture de l'entrée : {e}")))?;
        if ligne.trim().is_empty() {
            continue;
        }
        let reponse = match serde_json::from_str::<Value>(&ligne) {
            Ok(Value::Array(lot)) => {
                let r: Vec<Value> = lot.iter().filter_map(|m| repondre(racine, m)).collect();
                (!r.is_empty()).then_some(Value::Array(r))
            }
            Ok(m) => repondre(racine, &m),
            Err(e) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": format!("JSON illisible : {e}") } })),
        };
        if let Some(r) = reponse {
            let texte = serde_json::to_string(&r).unwrap_or_default();
            writeln!(sortie, "{texte}").and_then(|_| sortie.flush()).map_err(|e| Erreur::new(format!("écriture de la sortie : {e}")))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_cannot_leave_the_root() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("lots")).unwrap();
        let r = Racine::new(d.path()).unwrap();
        let racine = r.chemin().to_path_buf();
        assert_eq!(r.resoudre("lots/a.pdf").unwrap(), racine.join("lots").join("a.pdf"));
        assert_eq!(r.resoudre("lots/../b.pdf").unwrap(), racine.join("b.pdf"));
        assert_eq!(r.resoudre(&racine.join("c/d.pdf").to_string_lossy()).unwrap(), racine.join("c").join("d.pdf"));
        assert!(r.resoudre("../dehors.pdf").unwrap_err().message().contains("sort du dossier racine"));
        assert!(r.resoudre("lots/../../dehors.pdf").is_err());
        let ailleurs = tempfile::tempdir().unwrap();
        assert!(r.resoudre(&ailleurs.path().join("x.pdf").to_string_lossy()).is_err());
    }

    #[test]
    fn windows_verbatim_paths_read_the_usual_way() {
        assert_eq!(sans_prefixe_verbatim(r"\\?\C:\Users\<utilisateur>\PDF").as_deref(), Some(r"C:\Users\<utilisateur>\PDF"));
        assert_eq!(sans_prefixe_verbatim(r"\\?\UNC\serveur\partage\PDF").as_deref(), Some(r"\\serveur\partage\PDF"));
        assert_eq!(sans_prefixe_verbatim("/srv/pdf"), None);
        assert_eq!(sans_prefixe_verbatim(r"\\?\Volume{abc}\x"), None);
    }

    #[test]
    fn every_command_is_a_tool_with_a_schema() {
        let t = outils();
        let noms: Vec<&str> = t.iter().filter_map(|o| o["name"].as_str()).collect();
        for n in ["infos", "texte", "fusionner", "extraire", "decouper", "tourner", "supprimer", "deplacer", "inverser", "melanger", "ocr", "decouper-regex", "lister"] {
            assert!(noms.contains(&n), "{n} absent de {noms:?}");
        }
        assert!(!noms.contains(&"mcp"));
        let regex = t.iter().find(|o| o["name"] == "decouper-regex").unwrap();
        assert_eq!(regex["inputSchema"]["properties"]["regex"]["type"], "array");
        assert_eq!(regex["inputSchema"]["properties"]["ecraser"]["type"], "boolean");
        assert_eq!(regex["inputSchema"]["properties"]["ocr"]["enum"], json!(["auto", "toujours", "jamais"]));
        let requis = regex["inputSchema"]["required"].as_array().unwrap();
        assert!(requis.contains(&json!("fichier")) && requis.contains(&json!("regex")) && requis.contains(&json!("dossier")));
        let decouper = t.iter().find(|o| o["name"] == "decouper").unwrap();
        assert!(decouper["description"].as_str().unwrap().contains("Un paramètre parmi"));
    }
}
