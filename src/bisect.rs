use anyhow::{Context, Result};
use colored::*;
use dialoguer::Confirm;
use std::collections::HashSet;

use crate::snapshot::Snapshot;
use crate::package_diff::{compute_diff, PackageDiff, PackageChange};

pub struct BisectSession {
    good_snapshot: Snapshot,
    bad_snapshot: Snapshot,
    package_changes: Vec<PackageChange>,
    current_low: usize,
    current_high: usize,
    current_mid: usize,
    found_culprit: Option<PackageChange>,
}

impl BisectSession {
    pub fn new(good_snapshot: Snapshot, bad_snapshot: Snapshot) -> Result<Self> {
        let diff = compute_diff(&good_snapshot, &bad_snapshot)?;
        let package_changes = diff.all_changes();

        if package_changes.is_empty() {
            anyhow::bail!("No package changes detected between snapshots");
        }

        let total = package_changes.len();

        Ok(Self {
            good_snapshot,
            bad_snapshot,
            package_changes,
            current_low: 0,
            current_high: total,
            current_mid: total / 2,
            found_culprit: None,
        })
    }

    pub fn total_packages(&self) -> usize {
        self.package_changes.len()
    }

    pub fn run_manual(&mut self) -> Result<()> {
        let total_steps = (self.total_packages() as f64).log2().ceil() as usize;

        println!(
            "{} Binary search will take approximately {} steps",
            "ℹ️".cyan(),
            total_steps
        );
        println!();

        let mut step = 1;

        while self.current_low < self.current_high - 1 {
            println!(
                "{} {} ({}/{})",
                "Step".cyan().bold(),
                step,
                step,
                total_steps
            );
            println!();

            self.current_mid = (self.current_low + self.current_high) / 2;

            let test_packages: Vec<_> = self.package_changes[..self.current_mid]
                .iter()
                .collect();

            println!(
                "Testing with {}/{} packages installed...",
                test_packages.len(),
                self.total_packages()
            );
            println!();

            println!("{}", "Packages in this test:".dim());
            for pkg in test_packages.iter().take(10) {
                println!("  • {}", pkg.name().dim());
            }
            if test_packages.len() > 10 {
                println!("  ... and {} more", test_packages.len() - 10);
            }
            println!();

            println!("{}", "Please test your system now.".yellow().bold());
            println!("Boot into the snapshot and check if the issue occurs.");
            println!();

            let issue_occurs = Confirm::new()
                .with_prompt("Does the issue still occur?")
                .interact()?;

            println!();

            if issue_occurs {
                // Issue is in first half
                println!("{} Issue found in first half", "➡️".yellow());
                self.current_high = self.current_mid;
            } else {
                // Issue is in second half
                println!("{} Issue found in second half", "➡️".yellow());
                self.current_low = self.current_mid;
            }

            println!();
            step += 1;
        }

        // Found the culprit
        if self.current_low < self.package_changes.len() {
            let culprit = &self.package_changes[self.current_low];
            self.found_culprit = Some(culprit.clone());

            println!("{}", "🎯 FOUND THE CULPRIT!".green().bold());
            println!();
            println!("{} {}", "Package:".cyan(), culprit.name());

            match culprit {
                PackageChange::Added(pkg) => {
                    println!("{} Added (version {})", "Change:".cyan(), pkg.version);
                }
                PackageChange::Removed(pkg) => {
                    println!("{} Removed (was version {})", "Change:".cyan(), pkg.version);
                }
                PackageChange::Upgraded(pkg, old_ver, new_ver) => {
                    println!(
                        "{} Upgraded from {} to {}",
                        "Change:".cyan(),
                        old_ver,
                        new_ver
                    );
                }
                PackageChange::Downgraded(pkg, old_ver, new_ver) => {
                    println!(
                        "{} Downgraded from {} to {}",
                        "Change:".cyan(),
                        old_ver,
                        new_ver
                    );
                }
            }

            println!();
            println!("{}", "Recommended actions:".yellow());
            println!("  1. Downgrade just this package");
            println!("  2. Report issue to package maintainers");
            println!("  3. Check if others reported this issue");
            println!();
        }

        Ok(())
    }

    pub fn run_automated(&mut self) -> Result<()> {
        // Premium feature - automated testing with VMs
        println!("{}", "🤖 Automated Bisect (Premium)".cyan().bold());
        println!();

        println!("{}", "This feature will:".dim());
        println!("  • Boot test VMs for each bisect step");
        println!("  • Run your test suite automatically");
        println!("  • Find the culprit without manual intervention");
        println!();

        anyhow::bail!("Automated bisect requires Premium license");
    }
}
