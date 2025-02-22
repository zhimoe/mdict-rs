use actix_web::{web, Responder, Result};
use serde_derive::Deserialize;

use crate::lucky;
use crate::query::query;

#[derive(Deserialize, Debug)]
pub struct QueryForm {
    word: String,
}

pub(crate) async fn handle_query(params: web::Form<QueryForm>) -> Result<impl Responder> {
    Ok(query(params.word.clone()))
}

pub(crate) async fn handle_lucky() -> Result<impl Responder> {
    let word = lucky::lucky_word();
    Ok(query(word))
}
