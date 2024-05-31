use askama::Template;
use axum::{extract::Query, routing::get, Form, Router};
use axum_messages::{Message, Messages};
use serde::Deserialize;

use super::{app_state::AppState, next_url::*};

pub fn router(state: AppState) -> Router<()> {
    Router::new().route(
        "/register",
        get(self::get::register)
            .post(self::post::register)
            .with_state(state),
    )
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    messages: Vec<Message>,
    next: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub password2: String,
    pub next: Option<String>,
}

mod get {
    use super::*;

    pub async fn register(
        messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> RegisterTemplate {
        RegisterTemplate {
            messages: messages.into_iter().collect(),
            next,
        }
    }
}

mod post {
    use askama_axum::IntoResponse;
    use axum::{extract::State, response::Redirect};
    use axum_login::tracing::error;
    use http::StatusCode;

    use crate::web::app_state::AppState;

    use super::*;

    pub async fn register(
        messages: Messages,
        State(state): State<AppState>,
        Form(user): Form<RegisterForm>,
    ) -> impl IntoResponse {
        let mut url;

        if user.password != user.password2 {
            messages.error("pasword different");
            url = "/register".to_string();
        } else {
            match sqlx::query("select * from users where username = ? ")
                .bind(&user.username)
                .fetch_optional(&state.db)
                .await
            {
                Ok(u) => {
                    if u.is_some() {
                        messages.error("user name duplicate");
                        url = "/register".to_string();
                    } else {
                        match sqlx::query("insert into users (username, password) values (?, ?) ")
                            .bind(user.username)
                            .bind(user.password)
                            .execute(&state.db)
                            .await
                        {
                            Ok(_) => {
                                messages.success(format!("Successfully register, please login"));
                            }
                            Err(e) => {
                                error!("fail to register, error: {e}");
                                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            }
                        }
                        url = "/login".to_string();
                    }
                }
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
        }
        if let Some(next) = user.next {
            url = format!("{}?next={}", url, next);
        };

        return Redirect::to(&url).into_response();
    }
}
