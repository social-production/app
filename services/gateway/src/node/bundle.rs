//! What the gateway reads from the hApp bundle (ADR 0029): an integrity version made from
//! the roles' DNA hashes, and each role's coordinator zomes so they can be swapped in place.

use std::collections::BTreeMap;

use anyhow::{anyhow, Context};
use holochain_types::prelude::{
    AppBundle, CoordinatorBundle, CoordinatorManifest, DnaBundle, DnaManifest, DnaModifiersOpt,
    ZomeManifest,
};
use mr_bundle::Bundle;

#[derive(Debug)]
pub struct HappInfo {
    /// First 8 hex of sha256 over the sorted base DNA hashes: changes only with integrity.
    pub version: String,
    /// Role name → that role's coordinator zomes.
    pub coordinators: BTreeMap<String, CoordinatorBundle>,
}

pub async fn read(bundle: &[u8]) -> anyhow::Result<HappInfo> {
    let app = AppBundle::unpack(bundle).context("unpack hApp")?;
    let mut hashes = Vec::new();
    let mut coordinators = BTreeMap::new();
    for role in app.manifest().app_roles() {
        let path = role
            .dna
            .path
            .clone()
            .ok_or_else(|| anyhow!("role {} has no DNA", role.name))?;
        let bytes = app
            .get_resource(&path)
            .ok_or_else(|| anyhow!("missing DNA {path}"))?;
        let dna: DnaBundle = Bundle::unpack(bytes.as_ref())?.into();
        coordinators.insert(role.name.clone(), coordinators_of(&dna)?);
        let (_, hash) = dna.into_dna_file(DnaModifiersOpt::none()).await?;
        hashes.push(hash.to_string());
    }
    hashes.sort();
    Ok(HappInfo {
        version: crate::node::hosting::happ_version(hashes.join(",").as_bytes()),
        coordinators,
    })
}

fn coordinators_of(dna: &DnaBundle) -> anyhow::Result<CoordinatorBundle> {
    let DnaManifest::V0(manifest) = &dna.manifest().0;
    let zomes = manifest.coordinator.zomes.clone();
    let ids: Vec<String> = zomes.iter().map(ZomeManifest::resource_id).collect();
    let resources = dna
        .get_all_resources()
        .into_iter()
        .filter(|(id, _)| ids.contains(id))
        .map(|(id, bytes)| (id.clone(), bytes.clone()));
    Ok(Bundle::new(CoordinatorManifest { zomes }, resources)?.into())
}
