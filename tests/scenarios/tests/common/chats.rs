//! A chat clone for the conversation scenarios (ADR 0043): every conductor's agent has the
//! inbox (the base conversation cell) and the same chat.

use holo_hash::AgentPubKeyB64;
use holochain::sweettest::{SweetAgents, SweetCell, SweetConductorBatch};

/// Each conductor's agent gets the inbox (the base conversation cell) and the chat clone seeded
/// `<first agent>.<kind>.<nonce>`. Returns `(inbox, chat)` per conductor, and the seed.
pub async fn chat_network(
    conductors: &mut SweetConductorBatch,
    kind: &str,
) -> (Vec<(SweetCell, SweetCell)>, String) {
    let mut agents = Vec::new();
    for i in 0..conductors.len() {
        agents.push(SweetAgents::one(conductors[i].keystore()).await);
    }
    let creator = AgentPubKeyB64::from(agents[0].clone());
    let seed = format!("{creator}.{kind}.abcdefghijklmnopqrstuv");
    let inbox = super::load_dna("conversation").await;
    let chat = super::dna_with("conversation", seed.clone()).await;
    let mut cells = Vec::new();
    for (i, agent) in agents.into_iter().enumerate() {
        let app = conductors[i]
            .setup_app_for_agent("sp", agent, &[inbox.clone(), chat.clone()])
            .await
            .expect("install app");
        let mut both = app.into_cells();
        let chat_cell = both.remove(1);
        cells.push((both.remove(0), chat_cell));
    }
    conductors.exchange_peer_info().await;
    (cells, seed)
}
