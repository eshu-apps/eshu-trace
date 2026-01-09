# Eshu-Trace

**No More Rollbacks. Trace and Target the Exact Offending Package. Build On.**

Something broke after updates or new installs but you don't know which package caused it? Eshu-Trace uses binary search to find the culprit in ~6 steps instead of testing every package, hunting through logs, and wasting time on rollbacks.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Website](https://img.shields.io/badge/🌐-eshu--apps.com-blue)](https://eshu-apps.com)

**[💎 Get Eshu Trace - $19.99](https://gumroad.com/l/eshu-trace)** | **[🎁 Get Eshu Premium - $9.99/mo](https://eshuapps.gumroad.com/l/eshu-premium)** | **[💝 Donate](https://gumroad.com/l/eshu-donate)**

> 🎉 **3 FREE traces to try it out!**

## The Problem

```
Updated 47 packages yesterday →  System won't boot
  or ↓
Installed new software → GUI crashes
  or ↓
System update → Feature broken
```

**Which package did it?**

Traditional solutions waste time:
- ❌ **Full rollback** = Lose ALL updates, might break other things
- ❌ **Testing manually** = 47 packages = hours of work
- ❌ **Log hunting** = May not show the real culprit

## The Solution

**Eshu-Trace: Binary search to the rescue**

```bash
$ eshu-trace bisect

📦 47 packages changed
🔍 Binary search: ~6 steps instead of 47!

Step 1: Testing with 24/47 packages...
❓ Does issue occur? [y/n]: y

Step 2: Testing with 12/47 packages...
❓ Does issue occur? [y/n]: n

🎯 FOUND: nvidia-utils 545.29.06-1

What would you like to do?
  ⏪ Downgrade nvidia-utils to 545.23.06 (Recommended)
  📌 Pin version and prevent updates
  🗑️  Remove package
  ❌ Fix manually

[1] > 1

⏪ Downgrading nvidia-utils...
✓ Fixed! Reboot and verify.
```

## 🚨 System Won't Boot? No Problem!

**Eshu-Trace works from recovery mode, live USB, or old snapshots.**

### Option 1: Boot from Live USB (Easiest)
```bash
# 1. Boot Ubuntu/Arch/Fedora live USB
# 2. Mount your broken system
sudo mount /dev/sdXY /mnt

# 3. Install eshu-trace on the live USB
curl -L github.com/eshu-apps/eshu-trace/releases/latest/download/eshu-trace -o eshu-trace
chmod +x eshu-trace
sudo mv eshu-trace /usr/local/bin/

# 4. Run the trace - it auto-detects your mounted system!
sudo eshu-trace bisect
```

Eshu-Trace will:
- ✅ Detect you're in recovery mode
- ✅ Auto-find your mounted system
- ✅ Analyze the broken system
- ✅ Offer to fix it automatically

### Option 2: Boot into Recovery Mode
```bash
# 1. Restart → Hold SHIFT (GRUB)
# 2. Select "Recovery Mode" → "Root shell"
# 3. Run: eshu-trace bisect
```

### Option 3: Boot Old Snapshot (BTRFS/Timeshift)
```bash
# 1. Reboot → Select old snapshot from GRUB
# 2. System boots normally (from old state)
# 3. Run: eshu-trace bisect
# 4. It compares old vs new to find the breaker
```

**Full instructions:** `eshu-trace recovery`

## Pricing

### Try Free
**3 free traces** - No credit card required

### Then Choose

| Eshu-Trace | Eshu Premium |
|------------|--------------|
| **$19.99** one-time | **$9.99/mo** or $39/yr |
| ✓ Unlimited traces | ✓ Everything in Trace |
| ✓ Automatic fixes | ✓ Plus automated testing |
| ✓ Recovery mode support | ✓ Plus AI features |
| ✓ Works from live USB | ✓ Plus all installer features |
| [**Buy →**](https://eshu-trace.gumroad.com/l/eshu-trace) | [**Subscribe →**](https://eshu-installer.com/upgrade) |

**Need both tools?** Get Premium - it's cheaper than buying separately.

## What It Does

### 1. **Find the Breaking Package** (Binary Search)
- Tests ~6 combinations instead of all 47
- Works with any snapshot system (Timeshift, Snapper, BTRFS, LVM)
- Cross-distro (Arch, Debian, Fedora, etc.)

### 2. **Fix It Automatically**
After finding the culprit, Eshu-Trace offers:
- **Downgrade** to last working version (Recommended)
- **Pin version** to prevent future updates
- **Remove package** completely
- **Report bug** to maintainers

### 3. **Works on Broken Systems**
- Detects recovery mode automatically
- Works from chroot/live USB
- Finds your mounted system
- Applies fixes to the broken system

## Installation

### Quick Install
```bash
# Download
curl -L https://github.com/eshu-apps/eshu-trace/releases/latest/download/eshu-trace -o eshu-trace
chmod +x eshu-trace
sudo mv eshu-trace /usr/local/bin/

# Try it (3 free traces)
eshu-trace bisect
```

### After Purchase
```bash
eshu-trace activate --key YOUR_LICENSE_KEY --email you@email.com
```

Or if you have Eshu Premium, it auto-detects and gives unlimited access!

## Prerequisites

**Snapshot system** (one of):
- **Timeshift** (easiest) - `sudo pacman -S timeshift` or `sudo apt install timeshift`
- **Snapper** - `sudo pacman -S snapper`
- **BTRFS** snapshots
- **LVM** snapshots

## Usage

```bash
# Show recovery instructions (if system won't boot)
eshu-trace recovery

# Find breaking package
eshu-trace bisect

# List snapshots
eshu-trace snapshots

# Compare two snapshots
eshu-trace diff snapshot1 snapshot2

# Check trial status
eshu-trace status

# View purchase options
eshu-trace premium

# Activate license
eshu-trace activate
```

## Why Eshu-Trace?

### vs Manual Testing
- **47 packages** to test manually = Hours
- **Eshu-Trace** = 6 steps, ~10 minutes

### vs Full Rollback
- **Full rollback** = Lose all updates
- **Eshu-Trace** = Keep good updates, only fix the culprit

### vs Log Hunting
- **Logs** = May not show the real cause
- **Eshu-Trace** = Proves causation through bisection

## Technical Details

- **Built in Rust** - Fast, safe, single binary
- **Cross-distro** - Arch, Debian, Fedora, Gentoo, etc.
- **Recovery-aware** - Detects chroot, live USB, recovery mode
- **Automatic fixes** - Downgrade, pin, remove, report
- **Snapshot backends** - Timeshift, Snapper, BTRFS, LVM

## Integration with Eshu Installer (Premium Users)

If you have **Eshu Premium**, use Eshu-Trace directly from eshu-installer:

```bash
eshu trace bisect
eshu trace snapshots
eshu trace recovery
```

Your Premium license grants unlimited access!

## Support & Contact

- 🌐 **Website**: [eshu-apps.com](https://eshu-apps.com)
- 📧 **Support**: support@eshu-apps.com
- 📖 **Docs**: This README
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/eshu-apps/eshu-trace/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/eshu-apps/eshu-trace/discussions)
- 💝 **Donate**: [Support the Project](https://gumroad.com/l/eshu-donate)

## License

MIT License - see [LICENSE](LICENSE)

---

**Eshu** (Èṣù) is the Yoruba orisha of crossroads and problem-solving.
Eshu-Trace helps you solve package breakage without losing progress.
