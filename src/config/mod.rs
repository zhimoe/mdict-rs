use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tracing::info;

pub const MDX_FILES: &[&str] = &[
    "./resources/mdx/en/牛津高阶8.mdx",
    "./resources/mdx/en/朗文当代4.mdx",
    "./resources/mdx/zh/汉语词典3.mdx",
];

/// 获取项目root dir
pub fn static_path() -> anyhow::Result<PathBuf> {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.push("resources/static");
    Ok(path)
}

/// 全局数据库连接池，为每个MDX文件维护一个连接池 LazyLock会在第一次访问时初始化 此时indexing方法已完成，所以可以假定db file存在
pub static DB_POOLS: LazyLock<HashMap<String, Pool<SqliteConnectionManager>>> =
    LazyLock::new(|| {
        info!("initializing pools...");
        let mut pools = HashMap::new();

        for file in MDX_FILES {
            let db_file = format!("{file}.db");
            let manager = SqliteConnectionManager::file(&db_file).with_init(|conn| {
                // 设置SQLite性能优化参数
                conn.pragma_update(None, "journal_mode", "WAL").unwrap();
                conn.pragma_update(None, "synchronous", "NORMAL").unwrap();
                conn.pragma_update(None, "cache_size", "-64000").unwrap(); // 64MB cache
                conn.pragma_update(None, "busy_timeout", "5000").unwrap(); // 5 second busy timeout
                Ok(())
            });

            // 创建连接池，设置最大连接数为10，最小连接数为2
            let pool = Pool::builder()
                .max_size(10)
                .min_idle(Some(2))
                .build(manager)
                .unwrap_or_else(|_| panic!("Failed to create connection pool for {}", db_file));

            pools.insert(file.to_string(), pool);
        }

        pools
    });

/// 从连接池获取数据库连接
pub fn get_db_connection(
    file: &str,
) -> anyhow::Result<r2d2::PooledConnection<SqliteConnectionManager>> {
    info!("get connection from pool...");
    let pools = &*DB_POOLS;
    let pool = pools
        .get(file)
        .ok_or_else(|| anyhow::anyhow!("No connection pool found for file: {}", file))?;

    pool.get()
        .map_err(|e| anyhow::anyhow!("Failed to get connection from pool: {}", e))
}
