/*!
# Eshu Trace - Time Travel Debug for Linux Packages

Binary search through package update history to find the exact package that broke your system.

## How it works:
1. Detects package snapshots (uses system snapshot tools)
2. Finds package delta between working and broken states
3. Binary bisects through package changes
4. Identifies the culprit package

## Premium Features:
- Automated testing with VM/container boot
- AI-powered conflict prediction
- Community issue database integration
*/

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod bisect;
mod snapshot;
mod package_diff;
mod test_runner;
mod premium;

use crate::bisect::BisectSession;
use crate::snapshot::SnapshotManager;

#[derive(Parser)]
#[command(name = "eshu-trace")]
#[command(author = "Eshu Team")]
#[command(version)]
#[command(about = "Time Travel Debug - Find the package that broke your system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start bisect session to find problematic package
    Bisect {
        /// Snapshot ID when system was working
        #[arg(short, long)]
        good: Option<String>,

        /// Snapshot ID when system was broken
        #[arg(short, long)]
        bad: Option<String>,

        /// Automated testing (Premium)
        #[arg(long)]
        auto: bool,
    },

    /// List available snapshots
    Snapshots {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show package differences between snapshots
    Diff {
        /// First snapshot ID
        snapshot1: String,

        /// Second snapshot ID
        snapshot2: String,
    },

    /// Test if issue occurs with current packages
    Test {
        /// Test command to run
        #[arg(short, long)]
        command: Option<String>,
    },

    /// Show premium features and upgrade info
    Premium,

    /// Show status and configuration
    Status,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "✗ Error:".red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bisect { good, bad, auto } => {
            bisect_command(good, bad, auto)?;
        }
        Commands::Snapshots { verbose } => {
            list_snapshots(verbose)?;
        }
        Commands::Diff { snapshot1, snapshot2 } => {
            diff_command(snapshot1, snapshot2)?;
        }
        Commands::Test { command } => {
            test_command(command)?;
        }
        Commands::Premium => {
            show_premium_info()?;
        }
        Commands::Status => {
            show_status()?;
        }
    }

    Ok(())
}

fn bisect_command(good: Option<String>, bad: Option<String>, auto: bool) -> Result<()> {
    println!("{}", "🕐 Eshu Trace - Time Travel Debug".cyan().bold());
    println!();

    if auto && !premium::is_premium()? {
        println!("{}", "⚠️  Automated bisect is a Premium feature".yellow());
        println!("{}", "   Using manual bisect mode instead...".dim());
        println!();
    }

    let snapshot_mgr = SnapshotManager::new()?;

    // Detect snapshots
    let good_snapshot = if let Some(id) = good {
        snapshot_mgr.get_snapshot(&id)?
    } else {
        // Interactively select good snapshot
        snapshot_mgr.select_snapshot("Select snapshot when system was WORKING:")?
    };

    let bad_snapshot = if let Some(id) = bad {
        snapshot_mgr.get_snapshot(&id)?
    } else {
        // Interactively select bad snapshot
        snapshot_mgr.select_snapshot("Select snapshot when system was BROKEN:")?
    };

    println!();
    println!("{} {}", "Good snapshot:".green(), good_snapshot.id);
    println!("  Date: {}", good_snapshot.created_at);
    println!();
    println!("{} {}", "Bad snapshot:".red(), bad_snapshot.id);
    println!("  Date: {}", bad_snapshot.created_at);
    println!();

    // Start bisect session
    let mut session = BisectSession::new(good_snapshot, bad_snapshot)?;

    println!(
        "{} {} packages changed between snapshots",
        "📦".bold(),
        session.total_packages()
    );
    println!("{} Starting binary bisect...", "🔍".bold());
    println!();

    // Run bisect
    if auto && premium::is_premium()? {
        session.run_automated()?;
    } else {
        session.run_manual()?;
    }

    Ok(())
}

fn list_snapshots(verbose: bool) -> Result<()> {
    let snapshot_mgr = SnapshotManager::new()?;
    let snapshots = snapshot_mgr.list_snapshots()?;

    if snapshots.is_empty() {
        println!("{}", "No snapshots found".yellow());
        println!();
        println!("Create snapshots with your system's snapshot tool:");
        println!("  • Timeshift (BTRFS/rsync)");
        println!("  • Snapper (BTRFS)");
        println!("  • BTRFS snapshots");
        println!("  • LVM snapshots");
        return Ok(());
    }

    println!("{} Available Snapshots:", "📸".bold());
    println!();

    for snapshot in snapshots {
        println!("{} {}", "ID:".cyan(), snapshot.id);
        println!("   Date: {}", snapshot.created_at);

        if verbose {
            println!("   Packages: {}", snapshot.package_count.unwrap_or(0));

            if let Some(desc) = snapshot.description {
                println!("   Description: {}", desc);
            }
        }

        println!();
    }

    Ok(())
}

fn diff_command(snapshot1: String, snapshot2: String) -> Result<()> {
    let snapshot_mgr = SnapshotManager::new()?;

    let snap1 = snapshot_mgr.get_snapshot(&snapshot1)?;
    let snap2 = snapshot_mgr.get_snapshot(&snapshot2)?;

    println!("{} Package Differences", "📊".bold());
    println!();
    println!("{} {}", "Snapshot 1:".cyan(), snap1.id);
    println!("{} {}", "Snapshot 2:".cyan(), snap2.id);
    println!();

    let diff = package_diff::compute_diff(&snap1, &snap2)?;

    if !diff.added.is_empty() {
        println!("{} Added packages ({}):", "➕".green(), diff.added.len());
        for pkg in &diff.added {
            println!("   {} {}", "+".green(), pkg);
        }
        println!();
    }

    if !diff.removed.is_empty() {
        println!("{} Removed packages ({}):", "➖".red(), diff.removed.len());
        for pkg in &diff.removed {
            println!("   {} {}", "-".red(), pkg);
        }
        println!();
    }

    if !diff.upgraded.is_empty() {
        println!("{} Upgraded packages ({}):", "⬆️".yellow(), diff.upgraded.len());
        for (pkg, old_ver, new_ver) in &diff.upgraded {
            println!("   {} {} → {}", pkg, old_ver.dim(), new_ver);
        }
        println!();
    }

    if !diff.downgraded.is_empty() {
        println!("{} Downgraded packages ({}):", "⬇️".yellow(), diff.downgraded.len());
        for (pkg, old_ver, new_ver) in &diff.downgraded {
            println!("   {} {} → {}", pkg, old_ver.dim(), new_ver);
        }
        println!();
    }

    println!("Total changes: {}", diff.total_changes());

    Ok(())
}

fn test_command(command: Option<String>) -> Result<()> {
    println!("{}", "🧪 Testing for Issue".cyan().bold());
    println!();

    let test_cmd = if let Some(cmd) = command {
        cmd
    } else {
        dialoguer::Input::<String>::new()
            .with_prompt("Enter test command (or press Enter for interactive test)")
            .allow_empty(true)
            .interact()?
    };

    if test_cmd.is_empty() {
        println!("Run your test manually, then answer:");
        println!();
    } else {
        println!("Running: {}", test_cmd.cyan());
        println!();

        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(&test_cmd)
            .status()?;

        println!();

        if result.success() {
            println!("{} Test passed (exit code 0)", "✓".green());
        } else {
            println!(
                "{} Test failed (exit code {})",
                "✗".red(),
                result.code().unwrap_or(-1)
            );
        }

        return Ok(());
    }

    let issue_occurs = dialoguer::Confirm::new()
        .with_prompt("Does the issue still occur?")
        .interact()?;

    if issue_occurs {
        println!("{} Issue confirmed", "✗".red());
    } else {
        println!("{} Issue not present", "✓".green());
    }

    Ok(())
}

fn show_premium_info() -> Result<()> {
    println!("{}", "💎 Eshu Trace Premium".cyan().bold());
    println!();

    println!("{}", "Free Features:".green());
    println!("  ✓ Manual bisect (you test each step)");
    println!("  ✓ Snapshot comparison");
    println!("  ✓ Package diff viewer");
    println!();

    println!("{}", "Premium Features:".yellow());
    println!("  ⭐ Automated bisect (boots VMs, runs tests)");
    println!("  ⭐ AI conflict prediction");
    println!("  ⭐ Community issue database");
    println!("  ⭐ Automatic rollback creation");
    println!("  ⭐ Priority support");
    println!();

    println!("{}", "Pricing:".cyan());
    println!("  • Part of Eshu Premium subscription");
    println!("  • $9.99/month or $39.99/year");
    println!();

    println!("Upgrade at: https://eshu-installer.com/upgrade");
    println!("Support: https://github.com/sponsors/eshu-apps");

    Ok(())
}

fn show_status() -> Result<()> {
    println!("{}", "📊 Eshu Trace Status".cyan().bold());
    println!();

    // Check license
    let is_premium = premium::is_premium()?;
    let tier = if is_premium { "Premium" } else { "Free" };

    println!("{} {}", "License:".cyan(), tier);
    println!();

    // Check snapshot backend
    let snapshot_mgr = SnapshotManager::new()?;
    println!(
        "{} {}",
        "Snapshot backend:".cyan(),
        snapshot_mgr.backend_name()
    );
    println!(
        "{} {}",
        "Snapshots available:".cyan(),
        snapshot_mgr.list_snapshots()?.len()
    );
    println!();

    // System info
    println!("{}", "System Information:".cyan());

    if let Ok(output) = std::process::Command::new("uname").arg("-a").output() {
        if let Ok(info) = String::from_utf8(output.stdout) {
            println!("  {}", info.trim().dim());
        }
    }

    Ok(())
}
