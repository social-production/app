//! What happens when a new hApp is deployed (ADR 0028, ADR 0029): apps of other integrity
//! versions are removed (a new network), and coordinator-only changes are swapped into running
//! cells. Reinstalling instead would re-run genesis on an existing chain.

use std::collections::BTreeSet;

use holochain::conductor::api::{AdminRequest, AdminResponse, CellInfo};
use holochain_types::app::{CoordinatorSource, UpdateCoordinatorsPayload};

use super::Node;

impl Node {
    /// Remove hosted apps of other hApp versions (old networks) so their cells stop running.
    pub async fn remove_stale_apps(&self) -> anyhow::Result<usize> {
        let AdminResponse::AppsListed(apps) = self
            .admin(AdminRequest::ListApps {
                status_filter: None,
            })
            .await?
        else {
            return Ok(0);
        };
        let suffix = format!("-{}", self.happ_version);
        let mut removed = 0;
        for app in apps {
            let id = app.installed_app_id;
            let hosted = id.starts_with("person-") || id.starts_with("archivist-");
            let stale = hosted && !id.ends_with(&suffix);
            if stale || id == "sp-demo" || id.starts_with("warmup-") {
                self.admin(AdminRequest::UninstallApp {
                    installed_app_id: id,
                    force: true,
                })
                .await?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// Give every running cell of this hApp version the bundle's coordinators. Cheap when
    /// nothing changed (the WASM is cached by hash); returns how many DNAs were updated.
    pub async fn update_coordinators(&self) -> anyhow::Result<usize> {
        let AdminResponse::AppsListed(apps) = self
            .admin(AdminRequest::ListApps {
                status_filter: None,
            })
            .await?
        else {
            return Ok(0);
        };
        let mut done = BTreeSet::new();
        for app in apps
            .iter()
            .filter(|a| a.installed_app_id.ends_with(&self.happ_version))
        {
            for (role, cells) in &app.cell_info {
                let Some(bundle) = self.coordinators.get(role.as_str()) else {
                    continue;
                };
                for cell in cells {
                    let cell_id = match cell {
                        CellInfo::Provisioned(c) => c.cell_id.clone(),
                        CellInfo::Cloned(c) => c.cell_id.clone(),
                        CellInfo::Stem(_) => continue,
                    };
                    // Coordinators belong to the DNA, so one call covers every agent's cell.
                    if !done.insert(cell_id.dna_hash().clone()) {
                        continue;
                    }
                    let payload = UpdateCoordinatorsPayload {
                        cell_id,
                        source: CoordinatorSource::Bundle(Box::new(bundle.clone())),
                    };
                    self.admin(AdminRequest::UpdateCoordinators(Box::new(payload)))
                        .await?;
                }
            }
        }
        Ok(done.len())
    }
}
