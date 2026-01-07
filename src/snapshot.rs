use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: String,
    pub description: Option<String>,
    pub packages: Option<HashMap<String, String>>,
    pub package_count: Option<usize>,
}

pub struct SnapshotManager {
    backend: SnapshotBackend,
}

enum SnapshotBackend {
    Timeshift,
    Snapper,
    Btrfs,
    Lvm,
}

impl SnapshotManager {
    pub fn new() -> Result<Self> {
        let backend = Self::detect_backend()?;

        Ok(Self { backend })
    }

    fn detect_backend() -> Result<SnapshotBackend> {
        // Check for Timeshift
        if Command::new("which")
            .arg("timeshift")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(SnapshotBackend::Timeshift);
        }

        // Check for Snapper
        if Command::new("which")
            .arg("snapper")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(SnapshotBackend::Snapper);
        }

        // Check for BTRFS
        if std::path::Path::new("/.snapshots").exists() {
            return Ok(SnapshotBackend::Btrfs);
        }

        anyhow::bail!("No snapshot backend detected. Please install Timeshift, Snapper, or use BTRFS/LVM snapshots");
    }

    pub fn backend_name(&self) -> &str {
        match self.backend {
            SnapshotBackend::Timeshift => "Timeshift",
            SnapshotBackend::Snapper => "Snapper",
            SnapshotBackend::Btrfs => "BTRFS",
            SnapshotBackend::Lvm => "LVM",
        }
    }

    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        match self.backend {
            SnapshotBackend::Timeshift => self.list_timeshift_snapshots(),
            SnapshotBackend::Snapper => self.list_snapper_snapshots(),
            SnapshotBackend::Btrfs => self.list_btrfs_snapshots(),
            SnapshotBackend::Lvm => self.list_lvm_snapshots(),
        }
    }

    fn list_timeshift_snapshots(&self) -> Result<Vec<Snapshot>> {
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--list")
            .output()
            .context("Failed to run timeshift")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut snapshots = Vec::new();

        for line in stdout.lines() {
            if line.contains("@") && !line.starts_with('#') {
                let parts: Vec<&str> = line.split_whitespace().collect();

                if parts.len() >= 2 {
                    let id = parts[0].trim_start_matches('@').to_string();
                    let date = parts[1..].join(" ");

                    snapshots.push(Snapshot {
                        id: id.clone(),
                        created_at: date,
                        description: None,
                        packages: None,
                        package_count: None,
                    });
                }
            }
        }

        Ok(snapshots)
    }

    fn list_snapper_snapshots(&self) -> Result<Vec<Snapshot>> {
        let output = Command::new("sudo")
            .arg("snapper")
            .arg("list")
            .output()
            .context("Failed to run snapper")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut snapshots = Vec::new();

        for line in stdout.lines().skip(2) {
            // Skip header
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

            if parts.len() >= 5 {
                let id = parts[0].to_string();
                let date = parts[3].to_string();
                let description = if !parts[4].is_empty() {
                    Some(parts[4].to_string())
                } else {
                    None
                };

                snapshots.push(Snapshot {
                    id,
                    created_at: date,
                    description,
                    packages: None,
                    package_count: None,
                });
            }
        }

        Ok(snapshots)
    }

    fn list_btrfs_snapshots(&self) -> Result<Vec<Snapshot>> {
        // List snapshots in /.snapshots
        let snapshot_dir = std::path::Path::new("/.snapshots");

        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in std::fs::read_dir(snapshot_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        // Get metadata for creation time
                        if let Ok(metadata) = path.metadata() {
                            if let Ok(created) = metadata.created() {
                                let datetime: DateTime<Utc> = created.into();

                                snapshots.push(Snapshot {
                                    id: name_str.to_string(),
                                    created_at: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                                    description: None,
                                    packages: None,
                                    package_count: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(snapshots)
    }

    fn list_lvm_snapshots(&self) -> Result<Vec<Snapshot>> {
        // TODO: Implement LVM snapshot listing
        Ok(Vec::new())
    }

    pub fn get_snapshot(&self, id: &str) -> Result<Snapshot> {
        let snapshots = self.list_snapshots()?;

        snapshots
            .into_iter()
            .find(|s| s.id == id)
            .context(format!("Snapshot not found: {}", id))
    }

    pub fn select_snapshot(&self, prompt: &str) -> Result<Snapshot> {
        let snapshots = self.list_snapshots()?;

        if snapshots.is_empty() {
            anyhow::bail!("No snapshots available");
        }

        let items: Vec<String> = snapshots
            .iter()
            .map(|s| format!("{} - {}", s.id, s.created_at))
            .collect();

        let selection = dialoguer::Select::new()
            .with_prompt(prompt)
            .items(&items)
            .interact()?;

        Ok(snapshots[selection].clone())
    }
}
