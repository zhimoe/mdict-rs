use std::env;
use std::path::PathBuf;

use log::debug;

const MDX_PATH: &str = "resources/mdx/en/朗文当代4.mdx";

pub fn mdx_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.push(MDX_PATH.to_string());
    debug!("current path is {}", &path.display());
    Ok(path)
}

pub fn static_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.push("resources/static");
    Ok(path)
}
