use std::collections::HashMap;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // get /query?key=value
    // demonstrates an optional parameter.
    let query = warp::get()
        .and(warp::path("query"))
        .and(warp::query::<HashMap<String, String>>())
        .map(|p: HashMap<String, String>| match p.get("key") {
            Some(key) => Response::builder().body(format!("key = {}", key)),
            None => Response::builder().body(String::from("No \"key\" param in query.")),
        });

    let readme = warp::get()
        .and(warp::path("index1"))
        .and(warp::fs::file("static/index1.html"));

    // dir already requires GET...
    let examples = warp::path("test.css").and(warp::fs::file("static/test.css"));

    // GET / => README.md
    // GET /ex/... => ./examples/..
    let routes = readme.or(examples).or(query);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
