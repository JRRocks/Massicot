//! The executable's version, help and error messages.

use std::process::Command;

fn massicot(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_massicot")).args(args).output().expect("massicot runs")
}

#[test]
fn version_prints_the_delivery_number() {
    for flag in ["--version", "-V"] {
        let out = massicot(&[flag]);
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), format!("massicot {}", massicot::VERSION));
    }
}

#[test]
fn help_is_french() {
    for flag in ["--aide", "--help", "-h"] {
        let out = massicot(&[flag]);
        assert!(out.status.success(), "{flag}");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("Utilisation :"), "{text}");
        assert!(text.contains("Affiche la version"), "{text}");
        assert!(!text.contains("Usage:") && !text.contains("Print help"), "{text}");
    }
}

#[test]
fn unknown_argument_is_reported_in_french() {
    let out = massicot(&["--inconnu"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("argument inconnu : --inconnu"), "{err}");
    assert!(!err.contains("error:"), "{err}");
}

#[test]
fn welcome_of_the_start_menu_console() {
    let out = massicot(&["accueil"]);
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with(&format!("Massicot {} · boîte à outils PDF\n", massicot::VERSION)), "{text}");
    assert!(text.contains("massicot decouper-regex lot.pdf --regex \"PROCÈS-VERBAL N° (?<num>\\d+)\" --zone-haute 30\n"), "{text}");
    assert!(text.contains("décrit un PDF") && text.contains("faites glisser un PDF"), "{text}");
    assert!(text.ends_with("procès-verbal ».\n\n"), "a blank line before the prompt: {text}");
    assert!(!text.contains('\x1b'), "no colours when the output is not a console: {text}");
    // Hidden from the help: it is the shortcut's, not a command to learn.
    let aide = String::from_utf8_lossy(&massicot(&["--aide"]).stdout).to_string();
    assert!(!aide.contains("accueil"), "{aide}");
}
