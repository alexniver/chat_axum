use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use axum_messages::{Message, Messages};
use serde::Deserialize;

use super::next_url::*;
use super::users::AuthSession;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    messages: Vec<Message>,
    next: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/login", get(self::get::login).post(self::post::login))
        .route("/logout", get(self::get::logout))
}

mod get {
    use super::*;

    pub async fn login(
        messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> LoginTemplate {
        LoginTemplate {
            messages: messages.into_iter().collect(),
            next,
        }
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

mod post {
    use super::*;

    pub async fn login(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(login_creds): Form<LoginForm>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(login_creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                messages.error("Invalid credentials");

                let mut login_url = "/login".to_string();
                if let Some(next) = login_creds.next {
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

        if let Some(ref next) = login_creds.next {
            Redirect::to(next)
        } else {
            Redirect::to("/chat")
        }
        .into_response()
    }
}
