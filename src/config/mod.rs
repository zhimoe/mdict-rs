use std::env;
use std::path::PathBuf;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref MDX_FILES: Vec<&'static str> = vec![
        "./resources/mdx/en/牛津高阶10V3.mdx",
        // "./resources/mdx/en/牛津高阶8.mdx",
    ];
}

pub fn static_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.push("resources/static");
    Ok(path)
}
