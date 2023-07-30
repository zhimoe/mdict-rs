use std::env;
use std::path::PathBuf;

const MDX_PATH: &str = "resources/mdx/en/牛津高阶8.mdx";

pub fn mdx_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.push(MDX_PATH.to_string());
    println!("current path is {}", &path.display());
    Ok(path)
}

pub fn static_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    println!("current path is {}", &path.display());
    path.push("resources/static");
    Ok(path)
}
