use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::RwLock;
use warp::{filters::ws::WebSocket, Filter};

use crate::{
    main_server::ServerMessage,
    serial::write_serial,
    tracker::{TrackerData, TrackerInfo},
    MainServer,
};

pub const WEBSOCKET_PORT: u16 = 8298;

// Sent from server
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
enum WebsocketServerMessage {
    Error { error: String },
    TrackerInfo { info: TrackerInfo },
    TrackerData { index: usize, data: TrackerData },
}

// Receieved from client
#[derive(Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    Wifi { ssid: String, password: String },
    FactoryReset,
}

type WebsocketTx = Arc<RwLock<SplitSink<WebSocket, warp::ws::Message>>>;

async fn send_websocket_message(ws_tx: &WebsocketTx, message: WebsocketServerMessage) {
    if let Ok(string) = serde_json::to_string(&message) {
        let mut ws_tx = ws_tx.write().await;
        ws_tx.send(warp::ws::Message::text(string)).await.ok();
    }
}
pub async fn start_server(main: Arc<RwLock<MainServer>>) -> anyhow::Result<()> {
    let websocket = warp::ws()
        .and(warp::any().map(move || main.clone()))
        .map(|ws: warp::ws::Ws, main| ws.on_upgrade(|ws| on_connect(ws, main)));

    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT));
    log::info!("Started websocket server on {address}");
    warp::serve(websocket).run(address).await;
    Ok(())
}

async fn on_connect(ws: WebSocket, main: Arc<RwLock<MainServer>>) {
    log::info!("Websocket client connected");
    let (_ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(RwLock::new(_ws_tx));

    let (server_tx, mut server_rx) = tokio::sync::mpsc::unbounded_channel();
    main.write().await.message_channels.add(server_tx);

    for tracker in &main.read().await.trackers {
        send_websocket_message(
            &ws_tx,
            WebsocketServerMessage::TrackerInfo {
                info: tracker.info.clone(),
            },
        )
        .await;
    }

    // Spawn seperate task for listening to server messages
    let ws_tx_clone = ws_tx.clone();
    let server_messages_task = tokio::spawn(async move {
        while let Some(message) = server_rx.recv().await {
            handle_server_message(message, &ws_tx_clone).await;
        }
    });

    while let Some(ws_result) = ws_rx.next().await {
        let msg = match ws_result {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Websocket error: {e}");
                break;
            }
        };

        if let Ok(string) = msg.to_str() {
            log::info!("Got from websocket: {string}");
            if let Err(error) = handle_websocket_message(string) {
                log::error!("{error}");
                send_websocket_message(
                    &ws_tx,
                    WebsocketServerMessage::Error {
                        error: error.to_string(),
                    },
                )
                .await;
            }
        }
    }

    log::info!("Websocket client disconnected");
    server_messages_task.abort();
    server_messages_task.await.ok();
}

async fn handle_server_message(message: ServerMessage, ws_tx: &WebsocketTx) {
    match message {
        ServerMessage::TrackerInfoUpdate(info) => {
            send_websocket_message(ws_tx, WebsocketServerMessage::TrackerInfo { info }).await;
        }
        ServerMessage::TrackerDataUpdate((index, data)) => {
            send_websocket_message(ws_tx, WebsocketServerMessage::TrackerData { index, data })
                .await;
        }
    }
}

fn handle_websocket_message(string: &str) -> anyhow::Result<()> {
    let message = serde_json::from_str(string)?;

    match message {
        WebsocketClientMessage::Wifi { ssid, password } => {
            if ssid.len() > 32 || password.len() > 64 {
                return Err(anyhow::Error::msg("SSID or password too long"));
            }

            write_serial(format!("Wifi\0{ssid}\0{password}").as_bytes())?;
        }
        WebsocketClientMessage::FactoryReset => {
            write_serial("FactoryReset".as_bytes())?;
        }
    }

    Ok(())
}
