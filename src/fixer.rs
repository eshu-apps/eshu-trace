// Automatic package fixing after trace identifies culprit

use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Select};
use std::process::Command;

use crate::package_diff::PackageChange;
use crate::recovery::RecoveryContext;

pub struct PackageFixer {
    recovery_ctx: RecoveryContext,
}

#[derive(Debug)]
pub enum FixAction {
    Downgrade(String, String),      // package, target_version
    Remove(String),                  // package
    Pin(String, String),            // package, version
    ReportBug(String),              // package
    DoNothing,
}

impl PackageFixer {
    pub fn new(recovery_ctx: RecoveryContext) -> Self {
        Self { recovery_ctx }
    }

    pub fn offer_fix(&self, culprit: &PackageChange) -> Result<()> {
        println!();
        println!("{}", "═══════════════════════════════════════".green());
        println!("{} {}", "🎯 CULPRIT FOUND:".green().bold(), culprit.name());
        println!("{}", "═══════════════════════════════════════".green());
        println!();

        // Show what changed
        match culprit {
            PackageChange::Added(pkg) => {
                println!("  {} New package installed: {} {}", "➕".yellow(), pkg.name, pkg.version);
            }
            PackageChange::Removed(pkg) => {
                println!("  {} Package removed: {} {}", "➖".red(), pkg.name, pkg.version);
            }
            PackageChange::Upgraded(pkg, old_ver, new_ver) => {
                println!("  {} Package upgraded: {}", "⬆️".yellow(), pkg.name);
                println!("     From: {} → To: {}", old_ver.dim(), new_ver.yellow());
            }
            PackageChange::Downgraded(pkg, old_ver, new_ver) => {
                println!("  {} Package downgraded: {}", "⬇️".yellow(), pkg.name);
                println!("     From: {} → To: {}", old_ver.dim(), new_ver.yellow());
            }
        }

        println!();
        println!("{}", "What would you like to do?".cyan().bold());
        println!();

        // Present fix options
        let options = self.get_fix_options(culprit);
        let option_labels: Vec<String> = options.iter().map(|o| self.format_option(o)).collect();

        let selection = Select::new()
            .with_prompt("Choose action")
            .items(&option_labels)
            .default(0)
            .interact()?;

        // Execute chosen fix
        self.execute_fix(&options[selection], culprit)?;

        Ok(())
    }

    fn get_fix_options(&self, culprit: &PackageChange) -> Vec<FixAction> {
        let mut options = Vec::new();

        match culprit {
            PackageChange::Added(pkg) => {
                options.push(FixAction::Remove(pkg.name.clone()));
                options.push(FixAction::ReportBug(pkg.name.clone()));
            }
            PackageChange::Removed(pkg) => {
                // Can't easily re-add, suggest manual reinstall
                options.push(FixAction::ReportBug(pkg.name.clone()));
            }
            PackageChange::Upgraded(pkg, old_ver, _new_ver) => {
                options.push(FixAction::Downgrade(pkg.name.clone(), old_ver.clone()));
                options.push(FixAction::Pin(pkg.name.clone(), old_ver.clone()));
                options.push(FixAction::Remove(pkg.name.clone()));
                options.push(FixAction::ReportBug(pkg.name.clone()));
            }
            PackageChange::Downgraded(pkg, _old_ver, new_ver) => {
                options.push(FixAction::Pin(pkg.name.clone(), new_ver.clone()));
                options.push(FixAction::ReportBug(pkg.name.clone()));
            }
        }

        options.push(FixAction::DoNothing);
        options
    }

    fn format_option(&self, action: &FixAction) -> String {
        match action {
            FixAction::Downgrade(pkg, ver) => {
                format!("⏪ Downgrade {} to {} (Recommended)", pkg, ver)
            }
            FixAction::Remove(pkg) => {
                format!("🗑️  Remove {} completely", pkg)
            }
            FixAction::Pin(pkg, ver) => {
                format!("📌 Keep {} at {} and prevent future updates", pkg, ver)
            }
            FixAction::ReportBug(pkg) => {
                format!("🐛 Report bug for {} (opens issue)", pkg)
            }
            FixAction::DoNothing => {
                "❌ Do nothing (I'll fix it manually)".to_string()
            }
        }
    }

    fn execute_fix(&self, action: &FixAction, culprit: &PackageChange) -> Result<()> {
        match action {
            FixAction::Downgrade(pkg, version) => {
                self.downgrade_package(pkg, version)?;
            }
            FixAction::Remove(pkg) => {
                self.remove_package(pkg)?;
            }
            FixAction::Pin(pkg, version) => {
                self.pin_package(pkg, version)?;
            }
            FixAction::ReportBug(pkg) => {
                self.report_bug(pkg, culprit)?;
            }
            FixAction::DoNothing => {
                println!();
                println!("{} No changes made", "ℹ".cyan());
                println!("To fix manually:");
                println!("  • Check logs: journalctl -xe");
                println!("  • Search for similar issues");
                println!("  • Contact package maintainer");
            }
        }

        Ok(())
    }

    fn downgrade_package(&self, package: &str, version: &str) -> Result<()> {
        println!();
        println!("{} Downgrading {} to {}...", "⏪".yellow(), package, version);

        let distro = self.detect_distro()?;

        let chroot_prefix = if self.recovery_ctx.is_chroot {
            format!("arch-chroot {} ", self.recovery_ctx.system_root)
        } else {
            String::new()
        };

        let success = match distro.as_str() {
            "arch" | "manjaro" => {
                // Try pacman cache first
                let cmd = format!("{}sudo pacman -U /var/cache/pacman/pkg/{}-{}*.pkg.tar.*",
                                 chroot_prefix, package, version);

                println!("{} Running: {}", "→".dim(), cmd.dim());

                let result = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status()?;

                result.success()
            }
            "ubuntu" | "debian" => {
                let cmd = format!("{}sudo apt-get install {}={}", chroot_prefix, package, version);

                println!("{} Running: {}", "→".dim(), cmd.dim());

                let result = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status()?;

                result.success()
            }
            "fedora" | "rhel" => {
                let cmd = format!("{}sudo dnf downgrade {}-{}", chroot_prefix, package, version);

                println!("{} Running: {}", "→".dim(), cmd.dim());

                let result = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status()?;

                result.success()
            }
            _ => {
                println!("{} Unsupported distro for auto-downgrade", "⚠".yellow());
                return Ok(());
            }
        };

        if success {
            println!();
            println!("{} Successfully downgraded {}!", "✓".green().bold(), package);
            println!();
            println!("Next steps:");
            println!("  1. Reboot your system");
            println!("  2. Verify the issue is fixed");
            println!("  3. Consider pinning this version (see below)");
        } else {
            println!();
            println!("{} Downgrade failed", "✗".red());
            println!("You may need to:");
            println!("  • Clear package cache");
            println!("  • Download the old version manually");
            println!("  • Check if version {} exists", version);
        }

        Ok(())
    }

    fn remove_package(&self, package: &str) -> Result<()> {
        println!();

        if !Confirm::new()
            .with_prompt(format!("Really remove {}? This may break dependencies", package))
            .interact()? {
            return Ok(());
        }

        println!("{} Removing {}...", "🗑️".red(), package);

        let distro = self.detect_distro()?;
        let chroot_prefix = if self.recovery_ctx.is_chroot {
            format!("arch-chroot {} ", self.recovery_ctx.system_root)
        } else {
            String::new()
        };

        let cmd = match distro.as_str() {
            "arch" | "manjaro" => format!("{}sudo pacman -R {}", chroot_prefix, package),
            "ubuntu" | "debian" => format!("{}sudo apt-get remove {}", chroot_prefix, package),
            "fedora" | "rhel" => format!("{}sudo dnf remove {}", chroot_prefix, package),
            _ => {
                println!("{} Unsupported distro", "⚠".yellow());
                return Ok(());
            }
        };

        println!("{} Running: {}", "→".dim(), cmd.dim());

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()?;

        if result.success() {
            println!();
            println!("{} Successfully removed {}!", "✓".green().bold(), package);
        }

        Ok(())
    }

    fn pin_package(&self, package: &str, version: &str) -> Result<()> {
        println!();
        println!("{} Pinning {} at version {}...", "📌".yellow(), package, version);

        let distro = self.detect_distro()?;

        match distro.as_str() {
            "arch" | "manjaro" => {
                println!("Add to /etc/pacman.conf:");
                println!("  {}", format!("IgnorePkg = {}", package).yellow());
            }
            "ubuntu" | "debian" => {
                let cmd = format!("sudo apt-mark hold {}", package);
                println!("{} Running: {}", "→".dim(), cmd.dim());
                Command::new("sh").arg("-c").arg(&cmd).status()?;
                println!("{} Package pinned", "✓".green());
            }
            "fedora" | "rhel" => {
                println!("Add to /etc/dnf/dnf.conf:");
                println!("  {}", format!("exclude={}", package).yellow());
            }
            _ => {}
        }

        println!();
        println!("Package {} will not be updated automatically", package);
        println!("To unpin later, reverse these steps");

        Ok(())
    }

    fn report_bug(&self, package: &str, _culprit: &PackageChange) -> Result<()> {
        println!();
        println!("{} Generating bug report for {}...", "🐛".cyan(), package);
        println!();

        // Try to find package homepage/bug tracker
        let distro = self.detect_distro()?;

        let bug_url = match distro.as_str() {
            "arch" | "manjaro" => format!("https://bugs.archlinux.org/?project=0&string={}", package),
            "ubuntu" => format!("https://bugs.launchpad.net/ubuntu/+source/{}", package),
            "debian" => format!("https://bugs.debian.org/{}", package),
            "fedora" => format!("https://bugzilla.redhat.com/enter_bug.cgi?product=Fedora&component={}", package),
            _ => format!("https://github.com/search?q={}", package),
        };

        println!("Bug report information:");
        println!("  Package: {}", package.yellow());
        println!("  Issue: Package update caused system instability");
        println!("  Detected by: Eshu-Trace binary search");
        println!();
        println!("Report at: {}", bug_url.cyan());
        println!();
        println!("Opening in browser...");

        // Try to open browser
        let _ = Command::new("xdg-open").arg(&bug_url).spawn();

        Ok(())
    }

    fn detect_distro(&self) -> Result<String> {
        let os_release = if self.recovery_ctx.is_chroot {
            std::fs::read_to_string(format!("{}/etc/os-release", self.recovery_ctx.system_root))?
        } else {
            std::fs::read_to_string("/etc/os-release")?
        };

        for line in os_release.lines() {
            if line.starts_with("ID=") {
                let distro = line.trim_start_matches("ID=").trim_matches('"');
                return Ok(distro.to_string());
            }
        }

        Ok("unknown".to_string())
    }
}
