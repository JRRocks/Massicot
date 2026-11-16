//! The welcome text shown by the Start menu shortcut (`massicot accueil`), in a console that stays
//! open, until the window exists.

use std::io::IsTerminal;

/// One example line: the command, and what it does (grey) on the same line or the next one.
const EXEMPLES: [(&str, &str); 4] = [
    ("massicot infos lot.pdf", "décrit un PDF"),
    ("massicot fusionner a.pdf b.pdf --sortie ab.pdf", "fusionne deux PDF"),
    ("massicot decouper lot.pdf --toutes 2 --dossier parties", ""),
    (
        "massicot decouper-regex lot.pdf --regex \"PROCÈS-VERBAL N° (?<num>\\d+)\" --zone-haute 30",
        "un fichier par PV (mode à blanc d'abord)",
    ),
];

/// Column of the descriptions of the examples.
const COLONNE: usize = 52;

/// Text styles, empty when colours are off.
struct Styles {
    titre: &'static str,
    commande: &'static str,
    note: &'static str,
    fin: &'static str,
}

const COULEURS: Styles = Styles { titre: "\x1b[1;33m", commande: "\x1b[96m", note: "\x1b[90m", fin: "\x1b[0m" };
const SANS_COULEUR: Styles = Styles { titre: "", commande: "", note: "", fin: "" };

/// Colours only in Windows Terminal (or another terminal that says it understands them), never
/// when the output is redirected or `NO_COLOR` is set.
fn couleurs() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && (std::env::var_os("WT_SESSION").is_some() || std::env::var("TERM").is_ok_and(|t| t != "dumb"))
}

/// The welcome text, with colours when the console shows them.
pub fn texte() -> String {
    rendu(if couleurs() { &COULEURS } else { &SANS_COULEUR })
}

/// The welcome text without colours.
pub fn texte_simple() -> String {
    rendu(&SANS_COULEUR)
}

fn rendu(s: &Styles) -> String {
    let mut t = format!("{}Massicot {}{} · boîte à outils PDF\n\n", s.titre, crate::VERSION, s.fin);
    t.push_str("Tapez une commande ci-dessous, puis Entrée. Par exemple :\n\n");
    for (commande, note) in EXEMPLES {
        let largeur = 2 + commande.chars().count();
        t.push_str(&format!("  {}{commande}{}", s.commande, s.fin));
        if !note.is_empty() {
            if largeur < COLONNE {
                t.push_str(&" ".repeat(COLONNE - largeur));
            } else {
                t.push('\n');
                t.push_str(&" ".repeat(COLONNE));
            }
            t.push_str(&format!("{}{note}{}", s.note, s.fin));
        }
        t.push('\n');
    }
    t.push_str(&format!(
        "\nToutes les commandes : {c}massicot --aide{f}     Aide d'une commande : {c}massicot decouper --aide{f}\n",
        c = s.commande,
        f = s.fin
    ));
    t.push_str("Astuce : faites glisser un PDF dans cette fenêtre pour écrire son chemin.\n");
    t.push_str("Avec Claude Code, demandez simplement : « découpe lot.pdf, un fichier par procès-verbal ».\n");
    // A blank line before the console prompt, as on the validated preview.
    t.push('\n');
    t
}
