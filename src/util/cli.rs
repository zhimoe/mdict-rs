use anyhow::{anyhow, Context, Ok, Result};
use clap::Parser;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::{
    collections::HashMap, fs::read_dir, path::PathBuf, sync::LazyLock
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the dictionary storage directory. This directory should contain dictionary files used by the server.
    #[arg(short, long, value_name = "PATH")]
    dict_dir: Option<PathBuf>,

    /// Path to the static files directory. This directory serves as the root for site assets like HTML, CSS, and JavaScript.
    #[arg(short, long, value_name = "PATH")]
    static_dir: Option<PathBuf>,

    /// Generate the database files only, without starting the server.
    #[arg(short, long)]
    pub generate_only: bool,

    /// The port to bind the server to.
    #[arg(short, long, value_name = "PORT")]
    pub port: Option<u16>,

    /// The host to bind the server to.
    #[arg(long, value_name = "HOST")]
    pub host: Option<std::net::IpAddr>,
}

pub static ARGS: LazyLock<Cli> = LazyLock::new(|| Cli::parse());
pub static DB_POOLS: LazyLock<HashMap<String,Pool<SqliteConnectionManager>>> =
    LazyLock::new(|| create_pool().unwrap());

fn walk_dir(path: &PathBuf, dicts: &mut Vec<PathBuf>, ext: &str) -> Result<()> {
    for entry in
        read_dir(path).with_context(|| format!("Failed to read directory: {}", path.display()))?
    {
        let path = entry?.path();

        if path.is_file() {
            if let Some(e) = path.extension() {
                if e == ext {
                    dicts.push(path);
                }
            }
        } else if path.is_dir() {
            walk_dir(&path, dicts, ext)?;
        }
    }
    Ok(())
}

pub fn get_dicts_mdx() -> Result<Option<Vec<PathBuf>>> {
    let mut mdx = Vec::new();
    let path = get_dict_path().unwrap();
    walk_dir(&path, &mut mdx, "mdx")?;

    if mdx.is_empty() {
        Ok(None)
    } else {
        Ok(Some(mdx))
    }
}

fn get_dicts_db() -> Result<Vec<PathBuf>> {
    let mut db = Vec::new();
    let path = get_dict_path().unwrap();
    walk_dir(&path, &mut db, "db")?;
    if let Some(v) = get_dicts_mdx().unwrap() {
        for i in v {
            let db_file = i.with_extension("db");
            if !db.contains(&db_file) {
                db.push(db_file);
            }
        }
    }
    if db.is_empty() {
        return Err(anyhow!("No dictionary files found"));
    }
    Ok(db)
}

pub fn create_pool() -> Result<HashMap<String,Pool<SqliteConnectionManager>>> {
    let db_files = get_dicts_db().unwrap();
    let mut hashmap=HashMap::new();
    for path in db_files{
        let manager = SqliteConnectionManager::file(&path);
        let pool = Pool::new(manager).unwrap();
        hashmap.insert(path.file_stem().unwrap().to_string_lossy().to_string(), pool);
    }
    Ok(hashmap)
}

fn get_dict_path() -> Result<PathBuf> {
    let p = ARGS
        .dict_dir
        .as_ref()
        .filter(|i| i.exists() && i.is_dir())
        .cloned();
    if let Some(p) = p {
        Ok(p)
    } else {
        let p = PathBuf::from("resources/dict");
        if p.exists() && p.is_dir() {
            Ok(p)
        } else {
            Err(anyhow::anyhow!("dictionary directory not found"))
        }
    }
}

pub fn get_static_path() -> Result<PathBuf> {
    let p = ARGS
        .static_dir
        .as_ref()
        .filter(|i| i.exists() && i.is_dir())
        .cloned();
    if let Some(p) = p {
        Ok(p)
    } else {
        let p = PathBuf::from("resources/static");
        if p.exists() && p.is_dir() {
            Ok(p)
        } else {
            Err(anyhow::anyhow!("static directory not found"))
        }
    }
}
