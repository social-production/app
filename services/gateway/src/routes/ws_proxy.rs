//! `/ws`: Railway exposes one port per service, so the gateway forwards browser
//! websockets to the conductor's app interface on localhost. Frames pass through
//! untouched; authentication happens inside the Holochain protocol (app token).

use axum::extract::ws::{Message as AxumMsg, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as TMsg;

use crate::routes::AppState;

pub async fn upgrade(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    let port = state.config.app_ws_port;
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = pipe(socket, port).await {
            tracing::debug!(error = %e, "ws proxy closed");
        }
    })
}

async fn pipe(browser: WebSocket, port: u16) -> anyhow::Result<()> {
    let mut request = format!("ws://127.0.0.1:{port}").into_client_request()?;
    request
        .headers_mut()
        .insert("origin", "sp-gateway".parse()?);
    let (conductor, _) = tokio_tungstenite::connect_async(request).await?;

    let (mut to_browser, mut from_browser) = browser.split();
    let (mut to_conductor, mut from_conductor) = conductor.split();

    let upstream = async {
        while let Some(Ok(msg)) = from_browser.next().await {
            let Some(msg) = to_tungstenite(msg) else {
                break;
            };
            to_conductor.send(msg).await?;
        }
        anyhow::Ok(())
    };
    let downstream = async {
        while let Some(Ok(msg)) = from_conductor.next().await {
            let Some(msg) = to_axum(msg) else { break };
            to_browser.send(msg).await?;
        }
        anyhow::Ok(())
    };
    tokio::select! {
        result = upstream => result,
        result = downstream => result,
    }
}

fn to_tungstenite(msg: AxumMsg) -> Option<TMsg> {
    match msg {
        AxumMsg::Binary(b) => Some(TMsg::Binary(b)),
        AxumMsg::Text(t) => Some(TMsg::Text(t.as_str().into())),
        AxumMsg::Ping(p) => Some(TMsg::Ping(p)),
        AxumMsg::Pong(p) => Some(TMsg::Pong(p)),
        AxumMsg::Close(_) => None,
    }
}

fn to_axum(msg: TMsg) -> Option<AxumMsg> {
    match msg {
        TMsg::Binary(b) => Some(AxumMsg::Binary(b)),
        TMsg::Text(t) => Some(AxumMsg::Text(t.as_str().into())),
        TMsg::Ping(p) => Some(AxumMsg::Ping(p)),
        TMsg::Pong(p) => Some(AxumMsg::Pong(p)),
        TMsg::Close(_) | TMsg::Frame(_) => None,
    }
}
