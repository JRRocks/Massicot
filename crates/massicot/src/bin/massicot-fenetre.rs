//! `massicot-fenetre.exe`: the target of the Start menu shortcut. It starts `massicot.exe fenetre`
//! (the window lives in massicot.exe) without a console: as a graphical program, no console window
//! flashes when it is opened from the Start menu or the Explorer.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::Command;

fn main() {
    let nom = if cfg!(windows) { "massicot.exe" } else { "massicot" };
    let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join(nom))) else { return };
    let mut commande = Command::new(exe);
    commande.arg("fenetre").args(std::env::args_os().skip(1));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: massicot.exe runs without a console window.
        commande.creation_flags(0x0800_0000);
    }
    let _ = commande.spawn();
}
