//! The operations, shared by the command line and the MCP server. Each returns a serialisable
//! result (JSON for the MCP server and `--json`) that also renders as French text.

pub mod decoupe;
pub mod entetes;
pub mod fusion;
pub mod infos;
pub mod melange;
pub mod ocr;
pub mod pages;
pub mod sortie;

/// A size in bytes, written the French way ("1,2 Mo").
pub fn taille_lisible(octets: u64) -> String {
    const KO: f64 = 1024.0;
    let o = octets as f64;
    let texte = if o < KO {
        format!("{octets} octets")
    } else if o < KO * KO {
        format!("{:.0} ko", o / KO)
    } else if o < KO * KO * KO {
        format!("{:.1} Mo", o / KO / KO)
    } else {
        format!("{:.2} Go", o / KO / KO / KO)
    };
    texte.replace('.', ",")
}

#[cfg(test)]
mod tests {
    #[test]
    fn sizes_read_the_french_way() {
        assert_eq!(super::taille_lisible(512), "512 octets");
        assert_eq!(super::taille_lisible(350 * 1024), "350 ko");
        assert_eq!(super::taille_lisible(1_258_291), "1,2 Mo");
    }
}
