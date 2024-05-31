use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use axum_messages::{Message, Messages};

use super::next_url::*;
use super::users::AuthSession;

pub fn router() -> Router<()> {
    Router::new().route("/chat", get(self::get::chat).post(self::post::chat))
}

#[derive(Template)]
#[template(path = "chat.html")]
pub struct ChatTemplate {
    messages: Vec<Message>,
    next: Option<String>,
}

mod get {
    use super::*;

    pub async fn chat(messages: Messages, Query(NextUrl { next }): Query<NextUrl>) -> ChatTemplate {
        ChatTemplate {
            messages: messages.into_iter().collect(),
            next,
        }
    }
}

mod post {
    use super::*;

    pub async fn chat(mut auth_session: AuthSession, messages: Messages) -> impl IntoResponse {}
}
