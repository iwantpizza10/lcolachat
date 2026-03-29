use std::sync::Arc;

use axum::{Json, Router, extract::State, routing};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::Mutex};
use crate::{LcolachatWindow, MenuState};

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub author: String,
    pub content: String
}

struct ServerState {
    messages: Arc<Mutex<Vec<Message>>>,
    room_name: String
}

async fn get_slash(State(state): State<Arc<ServerState>>) -> String {
    state.room_name.clone()
}

async fn get_messages(State(state): State<Arc<ServerState>>) -> Json<Vec<Message>> {
    let messages = state.messages.lock().await;

    Json((*messages).clone().to_vec())
}

async fn add_message(State(state): State<Arc<ServerState>>, Json(body): Json<Message>) -> Json<Vec<Message>> {
    let mut messages = state.messages.lock().await;

    if body.content.trim().len() != 0 {
        (*messages).push(body);
    }

    Json((*messages).clone().to_vec())
}

pub async fn start_server(roomname: String, ui: LcolachatWindow) {
    let server_state = Arc::new(ServerState {
        messages: Arc::new(Mutex::new(vec![])),
        room_name: roomname
    });

    let server = Router::new()
        .route("/", routing::get(get_slash))
        .route("/messages", routing::get(get_messages))
        .route("/message", routing::post(add_message))
        .with_state(server_state);

    if let Ok(listener) = TcpListener::bind("0.0.0.0:3621").await {
        if let Err(_) = axum::serve(listener, server).await {
            ui.set_menu_state(MenuState::Error);
        }
    } else {
        ui.set_menu_state(MenuState::Error);
    }
}
