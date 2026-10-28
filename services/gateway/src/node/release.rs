//! What happens when a new hApp is deployed (ADR 0028, ADR 0029): apps of other integrity
//! versions are removed (a new network), and coordinator-only changes are swapped into running
//! cells. Reinstalling instead would re-run genesis on an existing chain.

use holochain::conductor::api::{AdminRequest, AdminResponse, AppInfo, CellInfo};
use holochain::prelude::DnaHash;
use holochain_types::app::{AppStatus, CoordinatorSource, UpdateCoordinatorsPayload};

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

    /// Installed apps, enabled apps and their cells, for `/readyz` (ADR 0064).
    pub async fn cell_counts(&self) -> anyhow::Result<serde_json::Value> {
        let AdminResponse::AppsListed(apps) = self
            .admin(AdminRequest::ListApps {
                status_filter: None,
            })
            .await?
        else {
            return Ok(serde_json::Value::Null);
        };
        let enabled = apps
            .iter()
            .filter(|a| matches!(a.status, AppStatus::Enabled))
            .count();
        let cells: usize = apps
            .iter()
            .map(|a| a.cell_info.values().map(Vec::len).sum::<usize>())
            .sum();
        Ok(serde_json::json!({ "apps": apps.len(), "enabled": enabled, "cells": cells }))
    }

    /// Give every cell of this hApp version the bundle's coordinators; returns how many cells
    /// were updated. Holochain 0.7 keeps a ribosome per cell, not per DNA, so each cell is
    /// updated on its own (ADR 0063).
    pub async fn update_coordinators(&self) -> anyhow::Result<usize> {
        let AdminResponse::AppsListed(apps) = self
            .admin(AdminRequest::ListApps {
                status_filter: None,
            })
            .await?
        else {
            return Ok(0);
        };
        let mut updated = 0;
        for app in apps
            .iter()
            .filter(|a| a.installed_app_id.ends_with(&self.happ_version))
        {
            updated += self.refresh_coordinators(app, None).await?;
        }
        Ok(updated)
    }

    /// Give one app's cells (only those of `dnas`, when given) the bundle's coordinators. A
    /// DNA whose integrity is unchanged keeps its hash, so a newly installed or cloned cell can
    /// start with the coordinator the conductor stored before (ADR 0063).
    pub async fn refresh_coordinators(
        &self,
        app: &AppInfo,
        dnas: Option<&[DnaHash]>,
    ) -> anyhow::Result<usize> {
        let mut updated = 0;
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
                if dnas.is_some_and(|d| !d.contains(cell_id.dna_hash())) {
                    continue;
                }
                let payload = UpdateCoordinatorsPayload {
                    cell_id,
                    source: CoordinatorSource::Bundle(Box::new(bundle.clone())),
                };
                self.admin(AdminRequest::UpdateCoordinators(Box::new(payload)))
                    .await?;
                updated += 1;
            }
        }
        Ok(updated)
    }
}
