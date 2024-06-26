use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::RwLock;
use warp::{filters::ws::WebSocket, Filter};

use crate::{main_server::ServerMessage, serial::write_serial, MainServer};

pub const WEBSOCKET_PORT: u16 = 8298;

// Receieved from client
#[derive(Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    Wifi { ssid: String, password: String },
    FactoryReset,
}

async fn send_websocket_message(
    ws_tx: &mut SplitSink<WebSocket, warp::ws::Message>,
    message: ServerMessage,
) {
    if let Ok(string) = serde_json::to_string(&message) {
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
    let (mut ws_tx, mut ws_rx) = ws.split();

    let mut server_rx = main.write().await.new_message_channel();

    for tracker in &main.read().await.trackers {
        send_websocket_message(
            &mut ws_tx,
            ServerMessage::TrackerInfo {
                info: tracker.info.clone(),
            },
        )
        .await;
    }

    // Spawn seperate task for listening to server messages
    let server_messages_task = tokio::spawn(async move {
        while let Some(message) = server_rx.recv().await {
            send_websocket_message(&mut ws_tx, message).await;
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
                main.write().await.notify_error(&error.to_string());
            }
        }
    }

    log::info!("Websocket client disconnected");
    server_messages_task.abort();
    server_messages_task.await.ok();
}

fn handle_websocket_message(message: &str) -> anyhow::Result<()> {
    match serde_json::from_str(message)? {
        WebsocketClientMessage::Wifi { ssid, password } => {
            if ssid.len() > 32 || password.len() > 64 {
                anyhow::bail!("SSID or password too long");
            }

            write_serial(format!("Wifi\0{ssid}\0{password}\n").as_bytes())?;
        }
        WebsocketClientMessage::FactoryReset => {
            write_serial(b"FactoryReset\n")?;
        }
    }

    Ok(())
}
