//! The MCP server spoken to as Claude does: JSON-RPC messages, one per line, on stdin/stdout.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use massicot_essais::{Ligne, Page, document, octets};
use serde_json::{Value, json};

struct Serveur {
    enfant: Child,
    entree: ChildStdin,
    sortie: BufReader<ChildStdout>,
    id: u64,
}

impl Serveur {
    fn lancer(racine: &std::path::Path) -> Serveur {
        let mut enfant = Command::new(env!("CARGO_BIN_EXE_massicot"))
            .args(["mcp", "--racine"])
            .arg(racine)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let entree = enfant.stdin.take().unwrap();
        let sortie = BufReader::new(enfant.stdout.take().unwrap());
        Serveur { enfant, entree, sortie, id: 0 }
    }

    fn envoyer(&mut self, message: Value) {
        writeln!(self.entree, "{message}").unwrap();
        self.entree.flush().unwrap();
    }

    fn demander(&mut self, methode: &str, params: Value) -> Value {
        self.id += 1;
        self.envoyer(json!({ "jsonrpc": "2.0", "id": self.id, "method": methode, "params": params }));
        let mut ligne = String::new();
        self.sortie.read_line(&mut ligne).unwrap();
        let reponse: Value = serde_json::from_str(&ligne).unwrap();
        assert_eq!(reponse["id"], self.id);
        reponse
    }

    fn outil(&mut self, nom: &str, args: Value) -> Value {
        self.demander("tools/call", json!({ "name": nom, "arguments": args }))["result"].clone()
    }
}

impl Drop for Serveur {
    fn drop(&mut self) {
        let _ = self.enfant.kill();
        let _ = self.enfant.wait();
    }
}

fn lot(dir: &std::path::Path) {
    let pages: Vec<Page> = [12, 13, 14]
        .iter()
        .flat_map(|n| {
            vec![
                Page::a4(vec![Ligne::new(60.0, 770.0, 18.0, format!("PROCÈS-VERBAL N° {n}")), Ligne::new(60.0, 720.0, 11.0, "Séance.")]),
                Page::a4(vec![Ligne::new(60.0, 760.0, 11.0, "Suite de la séance.")]),
            ]
        })
        .collect();
    std::fs::create_dir_all(dir.join("lots")).unwrap();
    std::fs::write(dir.join("lots/pv.pdf"), octets(&document(&pages, None, &[]))).unwrap();
}

#[test]
fn claude_can_list_tools_and_cut_a_batch_through_mcp() {
    let d = tempfile::tempdir().unwrap();
    lot(d.path());
    let mut s = Serveur::lancer(d.path());

    let init = s.demander("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "essai", "version": "1" } }));
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "massicot");
    assert!(init["result"]["instructions"].as_str().unwrap().contains("mode à blanc"));
    s.envoyer(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));

    let outils = s.demander("tools/list", json!({}));
    let noms: Vec<&str> = outils["result"]["tools"].as_array().unwrap().iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(noms.contains(&"decouper-regex") && noms.contains(&"fusionner") && noms.contains(&"ocr"), "{noms:?}");

    let liste = s.outil("lister", json!({}));
    assert_eq!(liste["structuredContent"]["fichiers"][0]["fichier"], "lots/pv.pdf");

    let infos = s.outil("infos", json!({ "fichier": "lots/pv.pdf" }));
    assert_eq!(infos["isError"], false);
    assert_eq!(infos["structuredContent"]["pages"], 6);

    let args = json!({
        "fichier": "lots/pv.pdf",
        "regex": [r"PROCÈS-VERBAL N° (?<num>\d+)"],
        "zone_haute": 30,
        "modele": "PV-{num}.pdf",
        "dossier": "sortie",
    });
    let blanc = s.outil("decouper-regex", args.clone());
    assert_eq!(blanc["isError"], false, "{blanc}");
    let rapport = &blanc["structuredContent"];
    assert_eq!(rapport["a_blanc"], true);
    let fichiers: Vec<&str> = rapport["blocs"].as_array().unwrap().iter().filter_map(|b| b["fichier"].as_str()).collect();
    assert_eq!(fichiers, ["PV-12.pdf", "PV-13.pdf", "PV-14.pdf"]);
    assert!(!d.path().join("sortie").exists());

    let mut coupe = args.clone();
    coupe["couper"] = rapport["jeton"].clone();
    let fait = s.outil("decouper-regex", coupe);
    assert_eq!(fait["isError"], false, "{fait}");
    assert!(d.path().join("sortie/PV-13.pdf").is_file());

    // Paths stay inside the root, and mistakes come back in French.
    let dehors = s.outil("infos", json!({ "fichier": "../ailleurs.pdf" }));
    assert_eq!(dehors["isError"], true);
    assert!(dehors["content"][0]["text"].as_str().unwrap().contains("sort du dossier racine"), "{dehors}");
    let mauvais = s.outil("decouper", json!({ "fichier": "lots/pv.pdf", "dossier": "x" }));
    assert_eq!(mauvais["isError"], true);
    assert!(mauvais["content"][0]["text"].as_str().unwrap().contains("argument obligatoire manquant"), "{mauvais}");
    let inconnu = s.outil("infos", json!({ "fichier": "lots/pv.pdf", "couleur": "rouge" }));
    assert!(inconnu["content"][0]["text"].as_str().unwrap().contains("paramètre « couleur » inconnu"), "{inconnu}");

    let ping = s.demander("ping", json!({}));
    assert_eq!(ping["result"], json!({}));
    let rien = s.demander("resources/list", json!({}));
    assert_eq!(rien["error"]["code"], -32601);
}
