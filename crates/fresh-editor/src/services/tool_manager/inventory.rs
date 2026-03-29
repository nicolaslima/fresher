//! Tool inventory: persistent JSON database of installed tools.

use anyhow::{Context, Result};
use fresh_core::api::InstalledToolInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// On-disk inventory format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFile {
    pub version: u32,
    pub tools: HashMap<String, ToolEntry>,
}

impl Default for InventoryFile {
    fn default() -> Self {
        Self {
            version: 1,
            tools: HashMap::new(),
        }
    }
}

/// A single tool entry in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub version: String,
    pub install_dir: String,
    pub installed_by: String,
    pub installed_at: String, // ISO 8601
    pub shim: Option<String>,
}

/// Manages the tool inventory JSON file.
pub struct ToolInventory {
    path: PathBuf,
    data: InventoryFile,
}

impl ToolInventory {
    /// Load the inventory from disk, or create a new empty one.
    pub fn load(tools_root: &Path) -> Result<Self> {
        let path = tools_root.join("inventory.json");
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read inventory at {}", path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse inventory at {}", path.display()))?
        } else {
            InventoryFile::default()
        };
        Ok(Self { path, data })
    }

    /// Save the inventory to disk.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let content =
            serde_json::to_string_pretty(&self.data).context("Failed to serialize inventory")?;
        std::fs::write(&self.path, content)
            .with_context(|| format!("Failed to write inventory to {}", self.path.display()))?;
        Ok(())
    }

    /// Register a new tool installation.
    pub fn register(
        &mut self,
        tool_name: &str,
        version: &str,
        install_dir: &Path,
        installed_by: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.data.tools.insert(
            tool_name.to_string(),
            ToolEntry {
                version: version.to_string(),
                install_dir: install_dir.to_string_lossy().to_string(),
                installed_by: installed_by.to_string(),
                installed_at: now,
                shim: Some(tool_name.to_string()),
            },
        );
        self.save()
    }

    /// Remove a tool from the inventory.
    pub fn remove(&mut self, tool_name: &str) -> Result<bool> {
        let removed = self.data.tools.remove(tool_name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Get all installed tools as `InstalledToolInfo`.
    pub fn list(&self) -> Vec<InstalledToolInfo> {
        let tools_bin = super::tools_bin_dir();
        self.data
            .tools
            .iter()
            .map(|(name, entry)| {
                let shim_path = entry
                    .shim
                    .as_ref()
                    .map(|s| tools_bin.join(s).to_string_lossy().to_string());
                InstalledToolInfo {
                    name: name.clone(),
                    version: entry.version.clone(),
                    install_dir: entry.install_dir.clone(),
                    installed_by: entry.installed_by.clone(),
                    installed_at: entry.installed_at.clone(),
                    shim_path,
                }
            })
            .collect()
    }

    /// Look up a single tool by name.
    pub fn get(&self, tool_name: &str) -> Option<&ToolEntry> {
        self.data.tools.get(tool_name)
    }
}
