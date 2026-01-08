// Recovery mode detection and chroot handling

use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct RecoveryContext {
    pub is_recovery: bool,
    pub is_chroot: bool,
    pub recovery_type: RecoveryType,
    pub system_root: String,
}

#[derive(Debug)]
pub enum RecoveryType {
    Normal,           // System booted normally
    LiveUSB,          // Running from live USB, system mounted
    Chroot,           // Inside chroot environment
    RecoveryMode,     // GRUB recovery/single-user mode
    SnapshotBoot,     // Booted into old snapshot
}

impl RecoveryContext {
    pub fn detect() -> Result<Self> {
        let is_chroot = Self::detect_chroot();
        let recovery_type = Self::detect_recovery_type(is_chroot);
        let system_root = Self::find_system_root(&recovery_type);

        Ok(Self {
            is_recovery: !matches!(recovery_type, RecoveryType::Normal),
            is_chroot,
            recovery_type,
            system_root,
        })
    }

    fn detect_chroot() -> bool {
        // Check if we're in a chroot by comparing root inode
        // In chroot, / inode != 2 (standard root inode)
        if let Ok(stat) = std::fs::metadata("/") {
            // In chroot or container, root inode is often different
            // This is a simple heuristic
            if Path::new("/proc/1/root").exists() {
                if let Ok(init_root) = std::fs::read_link("/proc/1/root") {
                    return init_root != Path::new("/");
                }
            }
        }

        // Check for arch-chroot or chroot markers
        Path::new("/.chroot_marker").exists() ||
        std::env::var("CHROOT").is_ok()
    }

    fn detect_recovery_type(is_chroot: bool) -> RecoveryType {
        // Check for live USB
        if Path::new("/run/archiso").exists() || // Arch live
           Path::new("/cdrom").exists() ||        // Ubuntu live
           Path::new("/lib/live").exists() {      // Debian live
            return RecoveryType::LiveUSB;
        }

        // Check for chroot
        if is_chroot {
            return RecoveryType::Chroot;
        }

        // Check for recovery mode (runlevel 1 or rescue.target)
        if let Ok(target) = Command::new("systemctl")
            .arg("get-default")
            .output() {
            let target_str = String::from_utf8_lossy(&target.stdout);
            if target_str.contains("rescue") || target_str.contains("emergency") {
                return RecoveryType::RecoveryMode;
            }
        }

        // Check if booted into snapshot
        if Self::is_snapshot_boot() {
            return RecoveryType::SnapshotBoot;
        }

        RecoveryType::Normal
    }

    fn is_snapshot_boot() -> bool {
        // Check if current boot is from a snapshot
        // BTRFS: check if mounted subvolume is a snapshot
        if let Ok(output) = Command::new("findmnt")
            .args(&["-n", "-o", "SOURCE", "/"])
            .output() {
            let source = String::from_utf8_lossy(&output.stdout);
            // Timeshift snapshots are in /@timeshift/snapshots/
            if source.contains("@timeshift") || source.contains("snapshots") {
                return true;
            }
        }
        false
    }

    fn find_system_root(recovery_type: &RecoveryType) -> String {
        match recovery_type {
            RecoveryType::LiveUSB | RecoveryType::Chroot => {
                // Try to find mounted system
                // Common mount points: /mnt, /mnt/arch, /mnt/gentoo, /target
                for path in &["/mnt", "/mnt/arch", "/mnt/gentoo", "/target", "/host"] {
                    if Path::new(path).join("etc/os-release").exists() {
                        return path.to_string();
                    }
                }
                "/mnt".to_string() // Default
            }
            _ => "/".to_string()
        }
    }

    pub fn show_recovery_banner(&self) {
        use colored::*;

        match self.recovery_type {
            RecoveryType::LiveUSB => {
                println!("{}", "╔════════════════════════════════════════╗".cyan());
                println!("{}", "║  RECOVERY MODE: Live USB Detected      ║".cyan());
                println!("{}", "╚════════════════════════════════════════╝".cyan());
                println!();
                println!("{} Your broken system is mounted at: {}", "✓".green(), self.system_root.yellow());
                println!("{} Eshu-Trace will analyze the mounted system", "ℹ".cyan());
                println!();
            }
            RecoveryType::Chroot => {
                println!("{}", "╔════════════════════════════════════════╗".cyan());
                println!("{}", "║  RECOVERY MODE: Chroot Environment     ║".cyan());
                println!("{}", "╚════════════════════════════════════════╝".cyan());
                println!();
                println!("{} Operating from chroot", "✓".green());
                println!("{} System root: {}", "ℹ".cyan(), self.system_root.yellow());
                println!();
            }
            RecoveryType::RecoveryMode => {
                println!("{}", "╔════════════════════════════════════════╗".cyan());
                println!("{}", "║  RECOVERY MODE: Safe Mode Boot         ║".cyan());
                println!("{}", "╚════════════════════════════════════════╝".cyan());
                println!();
            }
            RecoveryType::SnapshotBoot => {
                println!("{}", "╔════════════════════════════════════════╗".cyan());
                println!("{}", "║  RECOVERY MODE: Snapshot Boot          ║".cyan());
                println!("{}", "╚════════════════════════════════════════╝".cyan());
                println!();
                println!("{} Booted into old snapshot", "✓".green());
                println!("{} Will analyze differences to find breaking package", "ℹ".cyan());
                println!();
            }
            RecoveryType::Normal => {}
        }
    }

    pub fn ensure_mounted(&self) -> Result<()> {
        if matches!(self.recovery_type, RecoveryType::LiveUSB) {
            // Check if system is mounted
            if !Path::new(&self.system_root).join("etc/os-release").exists() {
                anyhow::bail!(
                    "System not mounted! Please mount your broken system first:\n\n\
                    For Arch/Manjaro:\n  \
                    sudo mount /dev/sdXY /mnt\n  \
                    sudo arch-chroot /mnt\n\n\
                    For Ubuntu/Debian:\n  \
                    sudo mount /dev/sdXY /mnt\n  \
                    sudo chroot /mnt\n\n\
                    Then run eshu-trace again."
                );
            }
        }
        Ok(())
    }
}

pub fn show_recovery_instructions() {
    use colored::*;

    println!();
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "  CAN'T BOOT? HERE'S HOW TO USE ESHU-TRACE FROM RECOVERY  ".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!();

    println!("{}", "OPTION 1: Boot from Live USB (Easiest)".yellow().bold());
    println!("  1. Boot from Ubuntu/Arch/Fedora live USB");
    println!("  2. Open terminal");
    println!("  3. Mount your broken system:");
    println!("     {}", "sudo mount /dev/sdXY /mnt".green());
    println!("     (Replace sdXY with your root partition)");
    println!();
    println!("  4. Install eshu-trace on the live USB:");
    println!("     {}", "curl -L github.com/eshu-apps/eshu-trace/releases/latest/download/eshu-trace -o eshu-trace".green());
    println!("     {}", "chmod +x eshu-trace".green());
    println!("     {}", "sudo mv eshu-trace /usr/local/bin/".green());
    println!();
    println!("  5. Run the trace:");
    println!("     {}", "sudo eshu-trace bisect".green());
    println!("     Eshu-Trace will auto-detect your mounted system!");
    println!();

    println!("{}", "OPTION 2: Boot into Recovery Mode".yellow().bold());
    println!("  1. Restart computer");
    println!("  2. Hold SHIFT (GRUB) or ESC (systemd-boot)");
    println!("  3. Select 'Advanced Options' → 'Recovery Mode'");
    println!("  4. Choose 'Drop to shell' or 'Root shell'");
    println!("  5. Run: {}", "eshu-trace bisect".green());
    println!();

    println!("{}", "OPTION 3: Boot into Old Snapshot (If using BTRFS/Timeshift)".yellow().bold());
    println!("  1. Reboot and select old snapshot from GRUB");
    println!("  2. System boots normally (from old state)");
    println!("  3. Run: {}", "eshu-trace bisect".green());
    println!("  4. It will compare old (working) vs new (broken)");
    println!();

    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!();
}
