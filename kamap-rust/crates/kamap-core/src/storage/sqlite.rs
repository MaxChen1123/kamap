use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::config::ProjectConfig;

/// SQLite 索引存储
pub struct IndexStore {
    conn: Connection,
}

impl IndexStore {
    /// 打开或创建索引数据库
    pub fn open(db_path: &Path) -> Result<Self> {
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open index db: {}", db_path.display()))?;

        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// 在内存中创建（用于测试）
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS file_index (
                id          INTEGER PRIMARY KEY,
                source_path TEXT NOT NULL,
                mapping_id  TEXT NOT NULL,
                asset_id    TEXT NOT NULL,
                priority    INTEGER DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_file_path ON file_index(source_path);

            CREATE TABLE IF NOT EXISTS range_index (
                id          INTEGER PRIMARY KEY,
                source_path TEXT NOT NULL,
                start_line  INTEGER NOT NULL,
                end_line    INTEGER NOT NULL,
                mapping_id  TEXT NOT NULL,
                asset_id    TEXT NOT NULL,
                segment     TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_range_path ON range_index(source_path);

            CREATE TABLE IF NOT EXISTS assets (
                id       TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                type     TEXT NOT NULL,
                target   TEXT NOT NULL,
                meta     TEXT
            );
            ",
        )?;
        Ok(())
    }

    /// 从配置重建索引
    pub fn rebuild(&self, config: &ProjectConfig) -> Result<()> {
        // 清空现有数据
        self.conn.execute_batch(
            "DELETE FROM file_index; DELETE FROM range_index; DELETE FROM assets;",
        )?;

        // 写入资产
        let mut asset_stmt = self.conn.prepare(
            "INSERT INTO assets (id, provider, type, target, meta) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for asset in &config.assets {
            let meta_json = serde_json::to_string(&asset.meta).unwrap_or_default();
            asset_stmt.execute(rusqlite::params![
                asset.id,
                asset.provider,
                asset.asset_type,
                asset.target,
                meta_json
            ])?;
        }

        // 写入映射索引
        let mut file_stmt = self.conn.prepare(
            "INSERT INTO file_index (source_path, mapping_id, asset_id, priority) VALUES (?1, ?2, ?3, ?4)",
        )?;
        let mut range_stmt = self.conn.prepare(
            "INSERT INTO range_index (source_path, start_line, end_line, mapping_id, asset_id, segment) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        for mapping in &config.mappings {
            if let Some(lines) = &mapping.source.lines {
                let segment_json = mapping
                    .segment
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap_or_default());
                range_stmt.execute(rusqlite::params![
                    mapping.source.path,
                    lines[0],
                    lines[1],
                    mapping.id,
                    mapping.asset,
                    segment_json
                ])?;
            } else {
                file_stmt.execute(rusqlite::params![
                    mapping.source.path,
                    mapping.id,
                    mapping.asset,
                    0
                ])?;
            }
        }

        Ok(())
    }

    /// 获取索引统计信息
    pub fn stats(&self) -> Result<IndexStats> {
        let file_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM file_index", [], |row| row.get(0))?;
        let range_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM range_index", [], |row| row.get(0))?;
        let asset_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM assets", [], |row| row.get(0))?;

        Ok(IndexStats {
            file_entries: file_count as usize,
            range_entries: range_count as usize,
            asset_entries: asset_count as usize,
        })
    }
}

/// 索引统计
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexStats {
    pub file_entries: usize,
    pub range_entries: usize,
    pub asset_entries: usize,
}
