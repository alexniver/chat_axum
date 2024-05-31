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
    Router::new().route("/", get(self::get::index))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    next: Option<String>,
}

mod get {
    use super::*;

    pub async fn index(Query(NextUrl { next }): Query<NextUrl>) -> IndexTemplate {
        IndexTemplate { next }
    }
}
