use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// to-ack.json 中的单条待确认记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToAckEntry {
    /// 映射 ID
    pub mapping_id: String,
    /// 资产 ID
    pub asset_id: String,
    /// 资产目标（如 docs/foo.md）
    pub asset_target: String,
    /// 触发变更的源文件
    pub source_path: String,
    /// 映射原因
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// 建议动作
    pub action: String,
    /// 是否已确认同步
    #[serde(default)]
    pub acked: bool,
}

/// to-ack.json 的完整结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToAckData {
    /// 本次 scan 绑定的 HEAD commit hash
    pub head_commit: String,
    /// 待确认条目
    pub items: Vec<ToAckEntry>,
}

/// to-ack.json 管理器
pub struct ToAckStore {
    path: PathBuf,
    data: Option<ToAckData>,
}

impl ToAckStore {
    /// 打开 to-ack.json（如果存在）
    pub fn open(workspace: &Path) -> Result<Self> {
        let path = workspace.join(".kamap").join("to-ack.json");

        let data = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            Some(serde_json::from_str::<ToAckData>(&content)
                .with_context(|| "Failed to parse to-ack.json")?)
        } else {
            None
        };

        Ok(Self { path, data })
    }

    /// 检查某个 mapping_id 是否已确认
    ///
    /// 只有 head_commit 匹配（代码没新 commit）且条目标记为 acked 时才返回 true
    pub fn is_acked(&self, mapping_id: &str, head_commit: &str) -> bool {
        match &self.data {
            Some(data) if data.head_commit == head_commit => {
                data.items.iter().any(|e| e.mapping_id == mapping_id && e.acked)
            }
            _ => false,
        }
    }

    /// 写入新的 scan 结果（替换整个文件）
    pub fn write_scan_result(&mut self, head_commit: &str, items: Vec<ToAckEntry>) -> Result<()> {
        self.data = Some(ToAckData {
            head_commit: head_commit.to_string(),
            items,
        });
        self.save()
    }

    /// 确认指定的 mapping_ids
    ///
    /// 返回 (成功确认数, 未找到的 ids)
    pub fn ack(&mut self, mapping_ids: &[String]) -> Result<(usize, Vec<String>)> {
        let data = self.data.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No scan results found. Run `kamap scan` first."))?;

        let mut acked_count = 0;
        let mut not_found = Vec::new();

        for id in mapping_ids {
            if let Some(entry) = data.items.iter_mut().find(|e| &e.mapping_id == id) {
                if !entry.acked {
                    entry.acked = true;
                    acked_count += 1;
                }
            } else {
                not_found.push(id.clone());
            }
        }

        self.save()?;
        Ok((acked_count, not_found))
    }

    /// 确认所有条目
    pub fn ack_all(&mut self) -> Result<usize> {
        let data = self.data.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No scan results found. Run `kamap scan` first."))?;

        let mut count = 0;
        for entry in &mut data.items {
            if !entry.acked {
                entry.acked = true;
                count += 1;
            }
        }

        self.save()?;
        Ok(count)
    }

    /// 获取当前数据（只读）
    pub fn data(&self) -> Option<&ToAckData> {
        self.data.as_ref()
    }

    /// 保存到文件
    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if let Some(data) = &self.data {
            let content = serde_json::to_string_pretty(data)
                .with_context(|| "Failed to serialize to-ack.json")?;
            std::fs::write(&self.path, content)
                .with_context(|| format!("Failed to write {}", self.path.display()))?;
        }
        Ok(())
    }
}
