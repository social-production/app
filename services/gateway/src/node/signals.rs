//! A hosted person's signals, heard in process (the vendored patch, ADR 0108).

use holochain::prelude::*;
use holochain_types::signal::Signal;
use tokio::sync::broadcast::Receiver;

use super::Node;

impl Node {
    /// Signals from the cells of the app `installed_app_id`.
    pub fn app_signals(&self, installed_app_id: &str) -> Receiver<Signal> {
        self.conductor.app_signals(installed_app_id.to_string())
    }

    /// Whether the person's app is installed here.
    pub async fn hosts(&self, agent: &AgentPubKey) -> anyhow::Result<bool> {
        let id = super::app_id(agent, &self.happ_version);
        Ok(self.find_app(&id).await?.is_some())
    }
}
