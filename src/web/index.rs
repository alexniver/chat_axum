use askama::Template;
use axum::{routing::get, Router};

pub fn router() -> Router<()> {
    Router::new().route("/", get(self::get::index))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

mod get {
    use super::*;

    pub async fn index() -> IndexTemplate {
        IndexTemplate
    }
}
