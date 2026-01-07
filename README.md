# Eshu Trace - Time Travel Debug for Linux

> Binary search through package update history to find the exact package that broke your system

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## The Problem

Something broke after updates but you don't know which package caused it:
- Updated 47 packages yesterday
- System now crashes on boot / app won't launch / feature broken
- Traditional rollback = lose ALL updates
- No way to find the culprit package

## The Solution

**Eshu Trace** uses binary search (bisect) to find the breaking package:

```bash
$ eshu-trace bisect

🕐 System working: 3 days ago (snapshot_20250103)
❌ System broken: now (snapshot_20250106)

📦 47 packages changed between snapshots
🔍 Starting binary bisect...

Testing with 24/47 packages installed...
❓ Does the issue still occur? [y/n]: y

Testing with 12/47 packages installed...
❓ Does the issue still occur? [y/n]: n

🎯 FOUND: Issue introduced by package 'nvidia-utils 545.29.06-1'

Would you like to:
1. Downgrade just this package
2. Report issue to maintainers
3. See if others reported this
```

## Pricing

**Eshu Trace is a paid product with a free trial.**

### ✨ Free Trial
- **3 free traces** to try before you buy
- Full feature access during trial
- No credit card required

### 💳 Purchase Options

**Option 1: Eshu Trace Standalone - $19.99 one-time**
- ✓ Unlimited traces
- ✓ Manual bisect
- ✓ Snapshot comparison
- ✓ Package diff viewer
- ✓ Priority email support
- [**Purchase →**](https://eshu-trace.gumroad.com/l/eshu-trace)

**Option 2: Eshu Premium - $9.99/month or $39.99/year** *(Best Value!)*
- ✓ **Everything** in Eshu Trace Standalone, PLUS:
- ⭐ Automated bisect (boots VMs, runs tests)
- ⭐ AI conflict prediction
- ⭐ Community issue database
- ⭐ **Full Eshu installer Premium features:**
  - Ghost Mode (`eshu try`)
  - Eshufile (system sync)
  - Conflict Oracle
  - AI-powered bundle suggestions
  - Unlimited AI queries
- ⭐ Priority support for all products
- [**Upgrade →**](https://eshu-installer.com/upgrade)

> **💡 Recommendation:** If you only need trace debugging → Eshu Trace Standalone ($19.99 one-time). If you also use eshu-installer → Eshu Premium ($9.99/mo) includes both!

## Features

### Included Features
- **Manual bisect** - Interactive binary search through package changes
- **Snapshot comparison** - View package differences between any snapshots
- **Package diff viewer** - See added/removed/upgraded packages
- **Multi-distro support** - Works with Arch, Debian, Fedora, and derivatives
- **Multiple snapshot backends** - Timeshift, Snapper, BTRFS, LVM

### Eshu Premium Exclusive ⭐
- **Automated bisect** - Boots VMs/containers and runs your test suite automatically
- **AI conflict prediction** - Predicts which packages might conflict
- **Community issue database** - See if others hit the same issue
- **Integration with eshu-installer** - Use `eshu trace` command

## Installation

### From Binary (Recommended)

```bash
# Download latest release
curl -L https://github.com/eshu-apps/eshu-trace/releases/latest/download/eshu-trace -o eshu-trace

# Make executable
chmod +x eshu-trace

# Move to PATH
sudo mv eshu-trace /usr/local/bin/
```

### From Source

```bash
# Clone repository
git clone https://github.com/eshu-apps/eshu-trace
cd eshu-trace

# Build with cargo
cargo build --release

# Install
sudo cp target/release/eshu-trace /usr/local/bin/
```

## Prerequisites

Eshu Trace requires a snapshot system:
- **Timeshift** (recommended) - `sudo pacman -S timeshift` or `sudo apt install timeshift`
- **Snapper** - `sudo pacman -S snapper` or `sudo apt install snapper`
- **BTRFS** snapshots
- **LVM** snapshots

## Quick Start

### 0. Activate Your License (After Purchase)

```bash
eshu-trace activate --key YOUR_LICENSE_KEY --email your@email.com
```

Or if you have Eshu Premium, it will automatically detect your license!

### 1. List Available Snapshots

```bash
eshu-trace snapshots
```

### 2. Compare Two Snapshots

```bash
eshu-trace diff snapshot_001 snapshot_002
```

### 3. Start Bisect

```bash
# Interactive mode
eshu-trace bisect

# Or specify snapshots directly
eshu-trace bisect --good snapshot_001 --bad snapshot_002
```

### 4. Follow the prompts

Eshu Trace will:
1. Calculate package delta between snapshots
2. Use binary search to narrow down the culprit
3. Test approximately `log2(N)` combinations
4. Identify the exact package that broke your system

## Usage

```
eshu-trace <COMMAND>

Commands:
  bisect      Start bisect session to find problematic package
  snapshots   List available snapshots
  diff        Show package differences between snapshots
  test        Test if issue occurs with current packages
  activate    Activate your license key
  premium     Show purchase options and pricing
  status      Show status and configuration
  help        Print this message or the help of the given subcommand(s)
```

### Trial Usage

Your first 3 traces are free! After that:
```bash
# See your trial status
eshu-trace status

# View purchase options
eshu-trace premium

# Activate license after purchase
eshu-trace activate
```

### Examples

```bash
# List all snapshots
eshu-trace snapshots --verbose

# Compare packages between two dates
eshu-trace diff snapshot_20250103 snapshot_20250106

# Find which package broke your system
eshu-trace bisect --good snapshot_20250103 --bad snapshot_20250106

# Test if issue still occurs
eshu-trace test --command "your-test-command"

# Check Premium status
eshu-trace premium
```

## How It Works

### Binary Search Algorithm

Instead of testing all 47 packages individually (47 steps), binary search tests:
1. First half (24 packages) → Issue present → Narrow to first half
2. First quarter (12 packages) → Issue absent → Narrow to second quarter
3. Continue bisecting...

Result: ~6 steps instead of 47!

### Snapshot Integration

Eshu Trace integrates with your existing snapshot system:
- Reads package lists from snapshot metadata
- Computes deltas between snapshots
- Guides you through testing each bisect step
- Never modifies your snapshots

## Integration with Eshu Installer (Premium Users)

If you have **Eshu Premium**, you can use eshu-trace directly from eshu-installer:

```bash
# Install via eshu-installer (Premium only)
eshu trace bisect
eshu trace snapshots
eshu trace diff snap1 snap2
```

Your Eshu Premium license automatically grants unlimited access to eshu-trace!

## Why Eshu Trace?

### Compared to Manual Debugging
- **47 packages** to check manually = Hours of work
- **Eshu Trace** = 6 steps, ~10 minutes

### Compared to Full Rollback
- **Full rollback** = Lose all updates, might break other things
- **Eshu Trace** = Keep good updates, only downgrade the culprit

### Compared to git bisect
- **git bisect** = For kernel/code commits
- **Eshu Trace** = For package changes (first of its kind!)

## Technical Details

### Architecture
- Written in **Rust** for performance and safety
- Zero-copy snapshot analysis
- Parallel package list parsing
- Minimal memory footprint

### Supported Package Managers
- pacman (Arch)
- dpkg/apt (Debian/Ubuntu)
- rpm/dnf (Fedora/RHEL)

### Supported Snapshot Systems
- Timeshift (BTRFS/rsync)
- Snapper (BTRFS)
- BTRFS native snapshots
- LVM snapshots

## Contributing

Contributions welcome! This is an open-source project.

```bash
# Fork the repo
git clone https://github.com/eshu-apps/eshu-trace
cd eshu-trace

# Create a branch
git checkout -b feature/your-feature

# Make changes and test
cargo test
cargo build --release

# Submit PR
```

## License

MIT License - see [LICENSE](LICENSE) for details

## Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/eshu-apps/eshu-trace/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/eshu-apps/eshu-trace/discussions)
- 💝 **Sponsor**: [GitHub Sponsors](https://github.com/sponsors/eshu-apps)
- 📧 **Email**: support@eshu-apps.com

## Roadmap

- [ ] Web UI for bisect visualization
- [ ] Integration with package manager logs
- [ ] Automatic issue reporting to upstream
- [ ] Support for flatpak/snap packages
- [ ] Cloud-based bisect (run tests in cloud VMs)
- [ ] Bisect history and analytics

## Credits

Created by the [Eshu Team](https://github.com/eshu-apps)

Inspired by git bisect but for Linux package management.

---

**Eshu** (Èṣù) is the Yoruba orisha of crossroads and messenger between worlds.
Eshu Trace helps you navigate the crossroads of package updates to find your way back to a working system.
