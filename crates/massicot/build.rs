//! Embeds the delivery version. The CI sets `MASSICOT_VERSION` (0.1.<run number>); a local build
//! says it is a development build.

fn main() {
    println!("cargo:rerun-if-env-changed=MASSICOT_VERSION");
    let version = match std::env::var("MASSICOT_VERSION") {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => format!("{}-dev", std::env::var("CARGO_PKG_VERSION").unwrap_or_default()),
    };
    println!("cargo:rustc-env=MASSICOT_VERSION={version}");
}
