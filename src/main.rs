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

    /// Activate license key
    Activate {
        /// License key from Gumroad
        #[arg(short, long)]
        key: Option<String>,

        /// Email address
        #[arg(short, long)]
        email: Option<String>,
    },

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
        Commands::Activate { key, email } => {
            activate_command(key, email)?;
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

    // Check license and trace limit
    let license = premium::get_license()?;

    if !license.can_trace() {
        println!("{}", "❌ Trial limit reached!".red().bold());
        println!();
        println!("You've used all {} free traces.", 3);
        println!();
        println!("{}", "Purchase Eshu Trace:".yellow());
        println!("  💳 Standalone license: {}", premium::get_upgrade_url());
        println!("  💎 Or get Eshu Premium (includes Trace): {}", premium::get_eshu_premium_url());
        println!();
        println!("{}", "Benefits of purchasing:".green());
        println!("  ✓ Unlimited traces");
        println!("  ✓ Automated bisect with VM testing");
        println!("  ✓ AI conflict prediction");
        println!("  ✓ Community issue database");
        println!("  ✓ Priority support");
        println!();
        anyhow::bail!("Trial limit reached. Please purchase a license to continue.");
    }

    // Show trial status
    match license.license_type {
        premium::LicenseType::Trial => {
            if let Some(remaining) = license.remaining_traces() {
                println!(
                    "{} Trial: {}/{} traces remaining",
                    "ℹ️".cyan(),
                    remaining,
                    3
                );
                println!("{}", "   Purchase: https://eshu-trace.gumroad.com/l/eshu-trace".dim());
                println!();
            }
        }
        premium::LicenseType::Standalone => {
            println!("{} Eshu Trace Licensed", "✓".green());
            println!();
        }
        premium::LicenseType::Premium => {
            println!("{} Eshu Premium (includes Trace)", "✓".green());
            println!();
        }
    }

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
    let result = if auto && premium::is_premium()? {
        session.run_automated()
    } else {
        session.run_manual()
    };

    // Increment usage count after successful trace
    if result.is_ok() {
        premium::increment_trace_usage()?;

        // Show updated trial status
        let license = premium::get_license()?;
        if license.license_type == premium::LicenseType::Trial {
            println!();
            if let Some(remaining) = license.remaining_traces() {
                if remaining > 0 {
                    println!(
                        "{} {} trial traces remaining",
                        "ℹ️".cyan(),
                        remaining
                    );
                    println!("{}", "   Purchase unlimited: https://eshu-trace.gumroad.com/l/eshu-trace".dim());
                } else {
                    println!("{}", "⚠️  This was your last free trace!".yellow().bold());
                    println!();
                    println!("Purchase Eshu Trace for unlimited traces:");
                    println!("  💳 {}", premium::get_upgrade_url());
                    println!("  💎 Or get Eshu Premium: {}", premium::get_eshu_premium_url());
                }
            }
        }
    }

    result
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
    println!("{}", "💎 Eshu Trace - Purchase Options".cyan().bold());
    println!();

    let license = premium::get_license()?;

    // Show current status
    match license.license_type {
        premium::LicenseType::Trial => {
            println!("{}", "Current Status: Trial".yellow());
            if let Some(remaining) = license.remaining_traces() {
                println!("Traces used: {}/3", license.traces_used);
                println!("Traces remaining: {}", remaining);
            }
            println!();
        }
        premium::LicenseType::Standalone => {
            println!("{}", "Current Status: Eshu Trace Licensed ✓".green());
            println!("Traces used: {} (unlimited)", license.traces_used);
            println!();
            return Ok(());
        }
        premium::LicenseType::Premium => {
            println!("{}", "Current Status: Eshu Premium ✓".green());
            println!("Traces used: {} (unlimited via Eshu Premium)", license.traces_used);
            println!();
            return Ok(());
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dim());
    println!();

    println!("{}", "OPTION 1: Eshu Trace Standalone".cyan().bold());
    println!();
    println!("{}", "What you get:".green());
    println!("  ✓ Unlimited traces");
    println!("  ✓ Manual bisect");
    println!("  ✓ Snapshot comparison");
    println!("  ✓ Package diff viewer");
    println!("  ✓ Priority email support");
    println!();
    println!("{}", "Pricing:".yellow());
    println!("  💳 $19.99 one-time payment");
    println!();
    println!("{}", "Purchase:".cyan());
    println!("  {}", premium::get_upgrade_url());
    println!();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dim());
    println!();

    println!("{}", "OPTION 2: Eshu Premium (Best Value!)".cyan().bold());
    println!();
    println!("{}", "What you get:".green());
    println!("  ✓ EVERYTHING in Eshu Trace Standalone, PLUS:");
    println!("  ⭐ Automated bisect (boots VMs, runs tests)");
    println!("  ⭐ AI conflict prediction");
    println!("  ⭐ Community issue database");
    println!("  ⭐ Full Eshu installer Premium features");
    println!("     • Ghost Mode (eshu try)");
    println!("     • Eshufile (system sync)");
    println!("     • Conflict Oracle");
    println!("     • AI-powered bundle suggestions");
    println!("     • Unlimited AI queries");
    println!("  ⭐ Priority support for all products");
    println!();
    println!("{}", "Pricing:".yellow());
    println!("  💎 $9.99/month or $39.99/year (save 33%)");
    println!();
    println!("{}", "Purchase:".cyan());
    println!("  {}", premium::get_eshu_premium_url());
    println!();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dim());
    println!();
    println!("{}", "💡 Recommendation:".yellow());
    println!("   If you only need trace → Eshu Trace ($19.99 one-time)");
    println!("   If you use eshu-installer too → Eshu Premium ($9.99/mo, includes both!)");

    Ok(())
}

fn activate_command(key: Option<String>, email: Option<String>) -> Result<()> {
    println!("{}", "🔑 Activate Eshu Trace License".cyan().bold());
    println!();

    let license_key = if let Some(k) = key {
        k
    } else {
        dialoguer::Input::<String>::new()
            .with_prompt("Enter your Gumroad license key")
            .interact()?
    };

    let email_addr = if let Some(e) = email {
        e
    } else {
        dialoguer::Input::<String>::new()
            .with_prompt("Enter your email address")
            .interact()?
    };

    println!();
    println!("{}", "Validating license...".dim());

    match premium::activate_license(&license_key, &email_addr) {
        Ok((true, message)) => {
            println!();
            println!("{} {}", "✓".green().bold(), message);
            println!();
            println!("{}", "Thank you for supporting Eshu Trace!".green());
            println!("You now have unlimited traces.");
        }
        Ok((false, message)) => {
            println!();
            println!("{} {}", "✗".red().bold(), message);
            println!();
            println!("Please check:");
            println!("  • License key is correct (copy-paste from Gumroad email)");
            println!("  • Email matches your purchase");
            println!();
            println!("Need help? Email: support@eshu-apps.com");
        }
        Err(e) => {
            println!();
            println!("{} Activation failed: {}", "✗".red().bold(), e);
            println!();
            println!("Need help? Email: support@eshu-apps.com");
        }
    }

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
