use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

use super::app_state::AppState;
use super::users::AuthSession;

pub fn router(app_state: AppState) -> Router<()> {
    Router::new()
        .route("/chat", get(self::get::chat))
        .route("/ws", get(self::get::ws_handler).with_state(app_state))
}

#[derive(Template)]
#[template(path = "chat.html")]
pub struct ChatTemplate;

#[derive(Template)]
#[template(path = "msg_chat.html")]
pub struct MsgChat {
    name: String,
    msg: String,
    is_me: bool,
}

impl MsgChat {
    fn new(name: String, msg: String) -> Self {
        Self {
            name,
            msg,
            is_me: false,
        }
    }
}

#[derive(Template)]
#[template(path = "msg_join.html")]
pub struct MsgJoin {
    msg: String,
}

impl MsgJoin {
    fn new(msg: String) -> Self {
        Self { msg }
    }
}

pub enum Msg {
    Chat(MsgChat),
    Join(MsgJoin),
}

mod get {
    use std::sync::Arc;

    use axum::extract::{
        ws::{self, WebSocket},
        State, WebSocketUpgrade,
    };
    use futures::{SinkExt, StreamExt};
    use tokio::sync::RwLock;

    use crate::web::{app_state::AppState, users::User};

    use super::*;

    pub async fn chat() -> ChatTemplate {
        ChatTemplate
    }

    pub async fn ws_handler(
        ws: WebSocketUpgrade,
        auth_session: AuthSession,
        State(state): State<AppState>,
    ) -> impl IntoResponse {
        if let Some(user) = auth_session.user {
            ws.on_upgrade(|socket| websocket(socket, user, state))
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }

    // This function deals with a single websocket connection, i.e., a single
    // connected client / user, for which we will spawn two independent tasks (for
    // receiving / sending chat messages).
    async fn websocket(stream: WebSocket, user: User, state: AppState) {
        // By splitting, we can send and receive at the same time.
        let (mut sender, mut receiver) = stream.split();

        // We subscribe *before* sending the "joined" message, so that we will also
        // display it to our client.
        let mut rx = state.tx.subscribe();

        let username = user.username;
        let name = username.clone();
        // Now send the "joined" message to all subscribers.
        let msg = format!("{} joined.", username);
        tracing::debug!("{msg}");
        let _ = state
            .tx
            .send(Arc::new(RwLock::new(Msg::Join(MsgJoin::new(msg)))));

        // Spawn the first task that will receive broadcast messages and send text
        // messages over the websocket to our client.
        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let mut msg = msg.write().await;
                match &mut *msg {
                    Msg::Chat(c) => {
                        c.is_me = c.name == name;
                        match c.render() {
                            Ok(msg) => {
                                // In any websocket error, break loop.
                                if sender.send(ws::Message::Text(msg)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("render chat template fail, e: {e}");
                                break;
                            }
                        }
                    }
                    Msg::Join(j) => match j.render() {
                        Ok(msg) => {
                            if sender.send(ws::Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("render join template fail, e: {e}");
                            break;
                        }
                    },
                }
            }
        });

        // Clone things we want to pass (move) to the receiving task.
        let tx = state.tx.clone();
        let name = username.clone();

        // Spawn a task that takes messages from the websocket, prepends the user
        // name, and sends them to all broadcast subscribers.
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(ws::Message::Text(text))) = receiver.next().await {
                // Add username before message.
                let _ = tx.send(Arc::new(RwLock::new(Msg::Chat(MsgChat::new(
                    name.clone(),
                    text,
                )))));
            }
        });

        // If any one of the tasks run to completion, we abort the other.
        tokio::select! {
            _ = &mut send_task => recv_task.abort(),
            _ = &mut recv_task => send_task.abort(),
        };

        // Send "user left" message (similar to "joined" above).
        let msg = format!("{username} left.");
        tracing::debug!("{msg}");
        let _ = state
            .tx
            .send(Arc::new(RwLock::new(Msg::Join(MsgJoin::new(msg)))));
    }
}
