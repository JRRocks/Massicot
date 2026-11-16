//! Massicot: a PDF toolbox driven from the command line, an MCP server and a window.
//!
//! User-facing messages are French; code and comments are English.

pub mod accueil;
pub mod cli;
pub mod commandes;
pub mod erreur;
pub mod fenetre;
pub mod lecture;
pub mod mcp;
pub mod outils;
pub mod ops;
pub mod pdf;

/// The delivery version (`0.1.<CI run number>`, or `<crate version>-dev` for a local build).
pub const VERSION: &str = env!("MASSICOT_VERSION");
