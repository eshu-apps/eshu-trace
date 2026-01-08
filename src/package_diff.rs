use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::process::Command;

use crate::snapshot::Snapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

#[derive(Debug, Clone)]
pub enum PackageChange {
    Added(Package),
    Removed(Package),
    Upgraded(Package, String, String), // package, old_version, new_version
    Downgraded(Package, String, String),
}

impl PackageChange {
    pub fn name(&self) -> &str {
        match self {
            PackageChange::Added(pkg) => &pkg.name,
            PackageChange::Removed(pkg) => &pkg.name,
            PackageChange::Upgraded(pkg, _, _) => &pkg.name,
            PackageChange::Downgraded(pkg, _, _) => &pkg.name,
        }
    }
}

#[derive(Debug)]
pub struct PackageDiff {
    pub added: Vec<Package>,
    pub removed: Vec<Package>,
    pub upgraded: Vec<(Package, String, String)>,
    pub downgraded: Vec<(Package, String, String)>,
}

impl PackageDiff {
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.upgraded.len() + self.downgraded.len()
    }

    pub fn all_changes(&self) -> Vec<PackageChange> {
        let mut changes = Vec::new();

        for pkg in &self.added {
            changes.push(PackageChange::Added(pkg.clone()));
        }

        for pkg in &self.removed {
            changes.push(PackageChange::Removed(pkg.clone()));
        }

        for (pkg, old_ver, new_ver) in &self.upgraded {
            changes.push(PackageChange::Upgraded(
                pkg.clone(),
                old_ver.clone(),
                new_ver.clone(),
            ));
        }

        for (pkg, old_ver, new_ver) in &self.downgraded {
            changes.push(PackageChange::Downgraded(
                pkg.clone(),
                old_ver.clone(),
                new_ver.clone(),
            ));
        }

        changes
    }
}

pub fn compute_diff(snapshot1: &Snapshot, snapshot2: &Snapshot) -> Result<PackageDiff> {
    let packages1 = get_packages_for_snapshot(snapshot1)?;
    let packages2 = get_packages_for_snapshot(snapshot2)?;

    let keys1: HashSet<_> = packages1.keys().collect();
    let keys2: HashSet<_> = packages2.keys().collect();

    // Added packages (in snapshot2, not in snapshot1)
    let added: Vec<Package> = keys2
        .difference(&keys1)
        .map(|name| Package {
            name: (*name).clone(),
            version: packages2[*name].clone(),
        })
        .collect();

    // Removed packages (in snapshot1, not in snapshot2)
    let removed: Vec<Package> = keys1
        .difference(&keys2)
        .map(|name| Package {
            name: (*name).clone(),
            version: packages1[*name].clone(),
        })
        .collect();

    // Version changes
    let mut upgraded = Vec::new();
    let mut downgraded = Vec::new();

    for name in keys1.intersection(&keys2) {
        let ver1 = &packages1[*name];
        let ver2 = &packages2[*name];

        if ver1 != ver2 {
            let pkg = Package {
                name: (*name).clone(),
                version: ver2.clone(),
            };

            // Simple version comparison (can be improved)
            if version_compare(ver2, ver1) {
                upgraded.push((pkg, ver1.clone(), ver2.clone()));
            } else {
                downgraded.push((pkg, ver1.clone(), ver2.clone()));
            }
        }
    }

    Ok(PackageDiff {
        added,
        removed,
        upgraded,
        downgraded,
    })
}

fn get_packages_for_snapshot(snapshot: &Snapshot) -> Result<HashMap<String, String>> {
    if let Some(ref packages) = snapshot.packages {
        return Ok(packages.clone());
    }

    // Detect package manager and get package list
    // This is a simplified version - in production, we'd read from snapshot filesystem
    detect_current_packages()
}

fn detect_current_packages() -> Result<HashMap<String, String>> {
    let mut packages = HashMap::new();

    // Try pacman first (Arch)
    if let Ok(output) = Command::new("pacman").arg("-Q").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.insert(parts[0].to_string(), parts[1].to_string());
                }
            }

            return Ok(packages);
        }
    }

    // Try dpkg (Debian/Ubuntu)
    if let Ok(output) = Command::new("dpkg").arg("-l").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                if line.starts_with("ii") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        packages.insert(parts[1].to_string(), parts[2].to_string());
                    }
                }
            }

            return Ok(packages);
        }
    }

    // Try rpm (Fedora/RHEL)
    if let Ok(output) = Command::new("rpm").arg("-qa").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                // Parse "package-name-version-release.arch"
                if let Some(pkg_info) = line.rsplitn(2, '-').nth(1) {
                    if let Some(name) = pkg_info.rsplitn(2, '-').nth(1) {
                        let version = line.strip_prefix(name).unwrap_or("").trim_start_matches('-');
                        packages.insert(name.to_string(), version.to_string());
                    }
                }
            }

            return Ok(packages);
        }
    }

    Ok(packages)
}

fn version_compare(v1: &str, v2: &str) -> bool {
    // Simple version comparison
    // In production, use a proper version comparison library

    let parts1: Vec<u32> = v1
        .split(&['.', '-', '_'][..])
        .filter_map(|s| s.parse().ok())
        .collect();

    let parts2: Vec<u32> = v2
        .split(&['.', '-', '_'][..])
        .filter_map(|s| s.parse().ok())
        .collect();

    for (a, b) in parts1.iter().zip(parts2.iter()) {
        if a > b {
            return true;
        } else if a < b {
            return false;
        }
    }

    parts1.len() > parts2.len()
}
