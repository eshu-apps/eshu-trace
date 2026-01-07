// Premium license checking with 3-free-traces trial

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const FREE_TRACE_LIMIT: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceLicense {
    pub license_key: Option<String>,
    pub license_type: LicenseType,
    pub email: Option<String>,
    pub activated_at: Option<String>,
    pub traces_used: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum LicenseType {
    Trial,           // 3 free traces
    Standalone,      // Paid eshu-trace license
    Premium,         // Part of eshu-installer Premium
}

impl Default for TraceLicense {
    fn default() -> Self {
        Self {
            license_key: None,
            license_type: LicenseType::Trial,
            email: None,
            activated_at: None,
            traces_used: 0,
        }
    }
}

impl TraceLicense {
    pub fn can_trace(&self) -> bool {
        match self.license_type {
            LicenseType::Trial => self.traces_used < FREE_TRACE_LIMIT,
            LicenseType::Standalone | LicenseType::Premium => true,
        }
    }

    pub fn remaining_traces(&self) -> Option<u32> {
        match self.license_type {
            LicenseType::Trial => {
                if self.traces_used < FREE_TRACE_LIMIT {
                    Some(FREE_TRACE_LIMIT - self.traces_used)
                } else {
                    Some(0)
                }
            }
            _ => None, // Unlimited for paid
        }
    }

    pub fn increment_usage(&mut self) {
        self.traces_used += 1;
    }
}

pub fn get_license() -> Result<TraceLicense> {
    let license_path = get_license_path();

    if !license_path.exists() {
        // Create default trial license
        let license = TraceLicense::default();
        save_license(&license)?;
        return Ok(license);
    }

    let data = fs::read_to_string(&license_path)
        .context("Failed to read license file")?;

    let license: TraceLicense = serde_json::from_str(&data)
        .context("Failed to parse license file")?;

    Ok(license)
}

pub fn save_license(license: &TraceLicense) -> Result<()> {
    let license_path = get_license_path();

    // Ensure directory exists
    if let Some(parent) = license_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(license)?;
    fs::write(&license_path, data)?;

    Ok(())
}

pub fn is_premium() -> Result<bool> {
    let license = get_license()?;
    Ok(license.license_type == LicenseType::Standalone
       || license.license_type == LicenseType::Premium)
}

pub fn check_can_trace() -> Result<bool> {
    let license = get_license()?;
    Ok(license.can_trace())
}

pub fn increment_trace_usage() -> Result<()> {
    let mut license = get_license()?;
    license.increment_usage();
    save_license(&license)?;
    Ok(())
}

pub fn activate_license(key: &str, email: &str) -> Result<(bool, String)> {
    // Validate license key with Gumroad
    if validate_gumroad_license(key, email)? {
        let mut license = get_license()?;
        license.license_key = Some(key.to_string());
        license.email = Some(email.to_string());
        license.license_type = LicenseType::Standalone;
        license.activated_at = Some(chrono::Utc::now().to_rfc3339());
        save_license(&license)?;

        Ok((true, "License activated successfully!".to_string()))
    } else {
        Ok((false, "Invalid license key".to_string()))
    }
}

fn validate_gumroad_license(key: &str, email: &str) -> Result<bool> {
    // Validate with Gumroad API
    // For now, simple validation (will be replaced with actual Gumroad API call)

    // Check if it's an Eshu Premium key (from eshu-installer)
    if is_eshu_premium_active()? {
        // If user has Eshu Premium, grant access
        return Ok(true);
    }

    // Check Gumroad license
    // This will be implemented with actual Gumroad API
    let product_permalink = "eshu-trace";
    let api_url = format!(
        "https://api.gumroad.com/v2/licenses/verify?product_permalink={}&license_key={}",
        product_permalink, key
    );

    // TODO: Implement actual HTTP request to Gumroad
    // For now, just check key format (will be replaced)
    Ok(key.starts_with("ESHU-TRACE-") && key.len() > 20)
}

fn is_eshu_premium_active() -> Result<bool> {
    // Check if user has active Eshu Premium (from eshu-installer)
    let eshu_license_path = get_eshu_installer_license_path();

    if !eshu_license_path.exists() {
        return Ok(false);
    }

    let data = fs::read_to_string(&eshu_license_path)?;
    let license_data: serde_json::Value = serde_json::from_str(&data)?;

    // Check if tier is premium and license is valid
    if let Some(tier) = license_data.get("tier") {
        if tier == "premium" {
            // Grant access via Eshu Premium
            let mut trace_license = get_license()?;
            if trace_license.license_type != LicenseType::Premium {
                trace_license.license_type = LicenseType::Premium;
                save_license(&trace_license)?;
            }
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_license_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".cache")
        .join("eshu-trace")
        .join("license.json")
}

fn get_eshu_installer_license_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".cache")
        .join("eshu")
        .join("license.json")
}

pub fn get_upgrade_url() -> &'static str {
    "https://eshu-trace.gumroad.com/l/eshu-trace"
}

pub fn get_eshu_premium_url() -> &'static str {
    "https://eshu-installer.com/upgrade"
}
