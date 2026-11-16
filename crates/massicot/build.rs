//! Embeds the delivery version. The CI sets `MASSICOT_VERSION` (0.1.<run number>); a local build
//! says it is a development build. On Windows, also embeds the icon and the version information
//! that Explorer shows (file properties).

fn main() {
    println!("cargo:rerun-if-env-changed=MASSICOT_VERSION");
    let version = match std::env::var("MASSICOT_VERSION") {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => format!("{}-dev", std::env::var("CARGO_PKG_VERSION").unwrap_or_default()),
    };
    println!("cargo:rustc-env=MASSICOT_VERSION={version}");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        ressources_windows(&version);
    }
}

/// Icon and version information of `massicot.exe` (Windows resources, compiled by `rc.exe`).
#[cfg(windows)]
fn ressources_windows(version: &str) {
    let manifeste = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by Cargo"));
    let icone = manifeste.join("../../assets/icone/massicot.ico");
    println!("cargo:rerun-if-changed={}", icone.display());

    // 0.1.12 or 0.1.0-dev -> 0.1.12.0 or 0.1.0.0
    let mut nombres = version.split(['.', '-']).map(|n| n.parse::<u64>().unwrap_or(0)).chain(std::iter::repeat(0));
    let numero = (0..4).fold(0u64, |acc, _| (acc << 16) | (nombres.next().unwrap_or(0) & 0xFFFF));

    let mut res = winresource::WindowsResource::new();
    res.set_icon(&icone.to_string_lossy())
        .set_language(0x040C) // French (France)
        .set_version_info(winresource::VersionInfo::FILEVERSION, numero)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, numero)
        .set("FileVersion", version)
        .set("ProductVersion", version)
        .set("ProductName", "Massicot")
        .set("FileDescription", "Massicot, boîte à outils PDF")
        .set("CompanyName", "Massicot")
        .set("LegalCopyright", "Licence MIT ou Apache-2.0")
        .set("OriginalFilename", "massicot.exe")
        .set("InternalName", "massicot");
    res.compile().expect("Windows resources (icon, version) compiled by rc.exe");
}

#[cfg(not(windows))]
fn ressources_windows(_version: &str) {
    println!("cargo:warning=build for Windows from another system: massicot.exe gets no icon");
}
