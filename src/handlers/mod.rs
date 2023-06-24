use std::arch::asm;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use serde_derive::Deserialize;

use crate::lucky;
use crate::query::query;

#[derive(Deserialize, Debug)]
pub struct QueryForm {
    word: String,
}

pub async fn handle_index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../resources/static/index.html")))
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
