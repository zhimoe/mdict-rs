use actix_web::{web, HttpResponse, Result};
use serde_derive::Deserialize;

use crate::lucky;
use crate::query::query;

#[derive(Deserialize, Debug)]
pub struct QueryForm {
    word: String,
}

pub async fn handle_query(params: web::Form<QueryForm>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("{}", query(params.word.clone()))))
}

pub async fn handle_lucky() -> Result<HttpResponse> {
    let word = lucky::lucky_word();
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("{}", query(word))))
}
