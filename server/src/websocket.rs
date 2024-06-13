use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{any, net::Ipv4Addr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    filters::ws::{Message, WebSocket},
    Filter,
};

use crate::{main_server::ServerMessage, serial::write_serial, tracker::TrackerInfo, MainServer};

pub const WEBSOCKET_PORT: u16 = 8298;

// Sent from server
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
enum WebsocketServerMessage {
    Error { error: String },
    TrackerInfo { info: TrackerInfo },
}

// Receieved from client
#[derive(Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    Wifi { ssid: String, password: String },
    FactoryReset,
}

async fn send_websocket_message(
    ws_tx: &Arc<RwLock<SplitSink<WebSocket, Message>>>,
    message: WebsocketServerMessage,
) {
    if let Ok(string) = serde_json::to_string(&message) {
        ws_tx.write().await.send(Message::text(string)).await.ok();
    }
}
pub async fn start_server(main: Arc<RwLock<MainServer>>) -> anyhow::Result<()> {
    let websocket = warp::ws()
        .and(warp::any().map(move || main.clone()))
        .map(|ws: warp::ws::Ws, main| ws.on_upgrade(|ws| on_connect(ws, main)));

    warp::serve(websocket)
        .run((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT))
        .await;

    Ok(())
}

async fn on_connect(ws: WebSocket, main: Arc<RwLock<MainServer>>) {
    log::info!("Websocket client connected");
    let (_ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(RwLock::new(_ws_tx));

    let (server_tx, mut server_rx) = tokio::sync::mpsc::unbounded_channel();
    main.write().await.add_message_channel(server_tx);

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
            match message {
                ServerMessage::TrackerInfoUpdate(info) => {
                    send_websocket_message(
                        &ws_tx_clone,
                        WebsocketServerMessage::TrackerInfo { info },
                    )
                    .await;
                }
                ServerMessage::TrackerDataUpdate(_) => todo!(),
            }
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

fn handle_websocket_message(string: &str) -> anyhow::Result<()> {
    let message = serde_json::from_str(string)?;

    match message {
        WebsocketClientMessage::Wifi { ssid, password } => {
            if ssid.len() > 32 || password.len() > 64 {
                return Err(anyhow::Error::msg("SSID or password too long"));
            }

            write_serial(format!("WIFI\0{ssid}\0{password}").as_bytes())?;
        }
        WebsocketClientMessage::FactoryReset => {
            write_serial("FACTORY-RESET".as_bytes())?;
        }
    }

    Ok(())
}
