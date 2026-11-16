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
