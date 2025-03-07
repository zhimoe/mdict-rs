use actix_web::{web, Either, HttpResponse, Responder, Result};
use serde_derive::Deserialize;

use crate::lucky;
use crate::query::query;
use crate::util::cli::FILE_MAP;

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

pub(crate) async fn handle_file(
    filename: web::Path<String>,
) -> Result<Either<actix_files::NamedFile, HttpResponse>, actix_web::Error> {
    let filename = filename.into_inner();
    // 当 FILE_MAP 为 None 时直接返回 404
    let Some(file_map) = &*FILE_MAP else {
        return Ok(Either::Right(
            HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Static resources not enabled"),
        ));
    };

    // 查找文件名映射
    let Some(real_path) = file_map.get(&filename) else {
        return Ok(Either::Right(HttpResponse::NotFound().finish()));
    };

    // 返回文件内容
    match actix_files::NamedFile::open_async(real_path).await {
        Ok(file) => Ok(Either::Left(file)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(Either::Right(HttpResponse::NotFound().finish()))
        }
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}
