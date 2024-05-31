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
use super::users::{AuthSession, Credentials};

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

    pub async fn chat(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                messages.error("Invalid credentials");

                let mut login_url = "/login".to_string();
                if let Some(next) = creds.next {
                    login_url = format!("{}?next={}", login_url, next);
                };

                return Redirect::to(&login_url).into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        messages.success(format!("Successfully logged in as {}", user.username));

        if let Some(ref next) = creds.next {
            Redirect::to(next)
        } else {
            Redirect::to("/")
        }
        .into_response()
    }
}
