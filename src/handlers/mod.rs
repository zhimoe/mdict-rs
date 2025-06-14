use crate::lucky;
use crate::query::query;
use serde_derive::Deserialize;

use axum::{extract::Form, response::Response};

#[derive(Deserialize, Debug)]
pub struct QueryForm {
    word: String,
}

pub(crate) async fn handle_query(Form(params): Form<QueryForm>) -> Response {
    let result = query(params.word);
    axum::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body(result.into())
        .unwrap()
}

pub(crate) async fn handle_lucky() -> Response {
    let word = lucky::lucky_word();
    let result = query(word);
    axum::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body(result.into())
        .unwrap()
}
