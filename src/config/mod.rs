use std::env;
use std::path::PathBuf;

pub const MDX_FILES: &[&str] = &[
    "./resources/mdx/en/牛津高阶8.mdx",
    "./resources/mdx/en/朗文当代4.mdx",
    "./resources/mdx/zh/汉语词典3.mdx",
];

pub fn static_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.push("resources/static");
    Ok(path)
}
