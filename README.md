# 💡 OpenCenter

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)
![Platform](https://img.shields.io/badge/Platform-Linux-blue.svg)
![License](https://img.shields.io/badge/License-MIT-green.svg)
![OpenDeck](https://img.shields.io/badge/StreamDeck-OpenAction%20Compatible-brightgreen.svg)

**OpenCenter** is a fast, lightweight Linux control center for **Elgato Key Lights**, **Key Light Air**, **Key Light Mini**, and **Light Strips**. Written in Rust, it provides a native GUI, system tray applet, CLI, mDNS auto-discovery, custom presets, smooth fading, and an OpenAction/OpenDeck plugin.

---

## ✨ Features

- 🔍 **mDNS Auto-Discovery**: Automatically discovers lights on the local network (`_elg._tcp.local`) with manual IP fallback.
- 🖥️ **Desktop GUI & Tray**: Dark-mode GUI (`eframe`) and DBus system tray applet (`ksni`) with live status and master sliders.
- ⚡ **CLI Automation**: Fast command-line interface for scripts, hotkeys, and macro pads.
- 🎮 **OpenDeck Plugin**: Native OpenAction integration for Stream Deck hardware buttons and dials.
- 🎨 **Presets & Smooth Fading**: One-click scenes (*Focus*, *Studio Call*, *Warm Reading*, *Night Shift*) with configurable fade transitions.

---

## 📦 Workspace Structure

| Crate | Description |
| :--- | :--- |
| **[`opencenter-core`](crates/opencenter-core)** | Shared library: Elgato REST client, mDNS discovery, models, and atomic config persistence. |
| **[`opencenter`](crates/opencenter-app)** | Main desktop binary: GUI window, system tray applet, and CLI parser. |
| **[`opencenter-opendeck`](crates/opencenter-opendeck)** | OpenAction plugin binary for OpenDeck and Stream Deck controllers. |

---

## 🛠️ Prerequisites

Install system dependencies for your Linux distribution:

```bash
# Ubuntu / Debian / Pop!_OS
sudo apt install build-essential pkg-config libdbus-1-dev libx11-dev libxkbcommon-dev

# Fedora / RHEL
sudo dnf install dbus-devel libX11-devel libxkbcommon-devel

# Arch Linux / Manjaro
sudo pacman -S base-devel dbus libx11 libxkbcommon
```

---

## ⚙️ Installation & Setup

### 1. Desktop App & Background Service
```bash
./install.sh

# Enable autostart on desktop login
systemctl --user enable --now opencenter
```

### 2. OpenDeck / Stream Deck Plugin
```bash
./package-opendeck.sh
```
- **In OpenDeck UI:** Click **Install Plugin** $\rightarrow$ select `opencenter-opendeck.zip`.
- **Manual install:** Copy `com.akitafuki.opencenter.sdPlugin` directly into `~/.local/share/OpenDeck/plugins/`.

**Plugin Actions:**
- `Toggle Lights`: Power toggle with real-time button state.
- `Apply Preset`: Triggers saved lighting scenes.
- `Adjust Brightness`: Step (+/- 10%) or set fixed brightness.
- `Adjust Temperature`: Step (+/- 500K) or set fixed Kelvin temperature.

---

## 🚀 Usage

### Desktop GUI & Tray
```bash
opencenter gui
```

### CLI Quick Reference

| Action | Command |
| :--- | :--- |
| **Auto-Discover** | `opencenter discover --save` |
| **Check Status** | `opencenter status` |
| **Toggle Power** | `opencenter toggle [IP / Group / all]` |
| **Set Power** | `opencenter on` / `opencenter off` |
| **Adjust Settings** | `opencenter set --brightness 80 --kelvin 4500` |
| **Smooth Fade** | `opencenter fade --brightness 100 --kelvin 5000 --duration-ms 2000` |
| **Apply Preset** | `opencenter preset apply "Studio Call"` |
| **Save Preset** | `opencenter preset save "Custom Scene"` |
| **Identify Light** | `opencenter identify <IP>` |
| **Add / Remove IP** | `opencenter add-ip <IP> --name "Desk"` / `opencenter remove-ip <IP>` |

---

## ⌨️ Global Hotkeys

Bind commands in **GNOME/KDE Settings $\rightarrow$ Keyboard Shortcuts**, or window manager configs:

```ini
# Hyprland (~/.config/hypr/hyprland.conf)
bind = SUPER ALT, L, exec, opencenter toggle
bind = SUPER ALT, S, exec, opencenter preset apply "Studio Call"
bind = SUPER ALT, N, exec, opencenter preset apply "Night Shift"

# Sway / i3 (~/.config/sway/config)
bindsym $mod+Mod1+l exec opencenter toggle
bindsym $mod+Mod1+s exec opencenter preset apply "Studio Call"
```

---

## 💾 Configuration

Settings are stored in `~/.config/opencenter/config.json`:

```json
{
  "devices": [
    { "ip": "192.168.1.50", "name": "Key Light Left", "enabled": true }
  ],
  "presets": [
    { "name": "Focus", "on": true, "brightness": 80, "kelvin": 5000 },
    { "name": "Studio Call", "on": true, "brightness": 100, "kelvin": 4500 }
  ]
}
```

---

## 📐 Technical API

Elgato lights communicate locally over plain HTTP JSON (**Port 9123**):
- `GET /elgato/accessory-info` – Metadata and serial numbers.
- `GET /elgato/lights` & `PUT /elgato/lights` – Power, brightness (0–100%), and color temperature in mireds ($M = \lfloor 1,000,000 / K \rfloor$, range: $2900\text{K} \leftrightarrow 344$, $7000\text{K} \leftrightarrow 143$).

---

## 🧪 Testing

```bash
cargo test --workspace
```

---

## 📄 License

[MIT License](LICENSE).
