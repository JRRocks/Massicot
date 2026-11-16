//! Command-line interface: argument parsing with French help and French error messages.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::error::{ContextKind, ErrorKind};
use clap::{ArgAction, Parser};

use crate::commandes::{Commande, executer};

/// Help layout of the main command (clap's own headings are English).
const ROOT_TEMPLATE: &str = "\
{name} {version} : {about}

Utilisation : {usage}

Commandes :
{subcommands}

Options :
{options}{after-help}";

#[derive(Debug, Parser)]
#[command(
    name = "massicot",
    version = crate::VERSION,
    about = "boîte à outils PDF (fusion, découpe, extraction, rotation, mélange, OCR, découpe par en-tête)",
    help_template = ROOT_TEMPLATE,
    override_usage = "massicot <COMMANDE> [options]",
    after_help = "\nAide d'une commande : massicot <COMMANDE> --aide",
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    subcommand_value_name = "COMMANDE",
    subcommand_help_heading = "Commandes"
)]
pub struct Cli {
    /// Affiche l'aide.
    #[arg(short = 'h', long = "aide", alias = "help", action = ArgAction::Help, global = true)]
    aide: Option<bool>,

    /// Affiche la version.
    #[arg(short = 'V', long = "version", action = ArgAction::Version)]
    version: Option<bool>,

    /// Résultat en JSON (pour un programme ou un agent) au lieu du texte.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    commande: Option<Commande>,
}

/// Entry point of the `massicot` executable.
pub fn main() -> ExitCode {
    run(std::env::args_os())
}

/// Parses `args` (program name first) and runs the command.
pub fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => return report_parse_error(&e),
    };
    let Some(commande) = cli.commande else {
        use clap::CommandFactory;
        let mut cmd = Cli::command();
        print!("{}", cmd.render_help());
        return ExitCode::SUCCESS;
    };
    match executer(&commande) {
        Ok((json, texte)) => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                print!("{texte}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            if cli.json {
                println!("{}", serde_json::json!({ "erreur": e.message() }));
            }
            eprintln!("massicot : {e}");
            ExitCode::from(1)
        }
    }
}

/// Prints help/version requests as they are, and argument errors in French.
fn report_parse_error(e: &clap::Error) -> ExitCode {
    match e.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            print!("{}", e.render());
            ExitCode::SUCCESS
        }
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            print!("{}", e.render());
            ExitCode::from(2)
        }
        _ => {
            eprintln!("massicot : {}", french_parse_error(e));
            eprintln!("Aide : massicot --aide, ou massicot <COMMANDE> --aide");
            ExitCode::from(2)
        }
    }
}

/// The French wording of a clap parsing error.
pub fn french_parse_error(e: &clap::Error) -> String {
    let ctx = |kind| e.get(kind).map(|v| v.to_string()).unwrap_or_default();
    let suggestion = |kind| match e.get(kind) {
        Some(v) => format!(" (vouliez-vous dire {v} ?)"),
        None => String::new(),
    };
    match e.kind() {
        ErrorKind::UnknownArgument => {
            format!("argument inconnu : {}{}", ctx(ContextKind::InvalidArg), suggestion(ContextKind::SuggestedArg))
        }
        ErrorKind::InvalidSubcommand => format!(
            "commande inconnue : {}{}",
            ctx(ContextKind::InvalidSubcommand),
            suggestion(ContextKind::SuggestedSubcommand)
        ),
        ErrorKind::MissingRequiredArgument => format!("argument obligatoire manquant : {}", ctx(ContextKind::InvalidArg)),
        ErrorKind::MissingSubcommand => "commande manquante".to_string(),
        ErrorKind::InvalidValue => {
            let valid = ctx(ContextKind::ValidValue);
            let mut s = format!("valeur « {} » invalide pour {}", ctx(ContextKind::InvalidValue), ctx(ContextKind::InvalidArg));
            if !valid.is_empty() {
                s.push_str(&format!(" (valeurs possibles : {valid})"));
            }
            s
        }
        ErrorKind::ValueValidation => {
            let cause = std::error::Error::source(e).map(|c| format!(" : {c}")).unwrap_or_default();
            format!("valeur « {} » invalide pour {}{cause}", ctx(ContextKind::InvalidValue), ctx(ContextKind::InvalidArg))
        }
        ErrorKind::ArgumentConflict => {
            format!("{} ne peut pas être utilisé avec {}", ctx(ContextKind::InvalidArg), ctx(ContextKind::PriorArg))
        }
        ErrorKind::TooManyValues | ErrorKind::TooFewValues | ErrorKind::WrongNumberOfValues => {
            format!("nombre de valeurs incorrect pour {}", ctx(ContextKind::InvalidArg))
        }
        ErrorKind::NoEquals => format!("{} attend « = » suivi d'une valeur", ctx(ContextKind::InvalidArg)),
        ErrorKind::InvalidUtf8 => "un argument contient des caractères illisibles (UTF-8 invalide)".to_string(),
        _ => "arguments invalides".to_string(),
    }
}
