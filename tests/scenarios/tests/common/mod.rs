use std::path::PathBuf;

use holochain::sweettest::SweetDnaFile;

/// Load a packed DNA from `dnas/<name>/sp_<name>.dna` (built by scripts/build-happ.sh).
pub async fn load_dna(name: &str) -> holochain::prelude::DnaFile {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../dnas")
        .join(name)
        .join(format!("sp_{name}.dna"));
    SweetDnaFile::from_bundle(&path)
        .await
        .unwrap_or_else(|e| panic!("load {}: {e} — run scripts/build-happ.sh", path.display()))
}
