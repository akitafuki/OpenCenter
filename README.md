# 💡 OpenCenter

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)
![Platform](https://img.shields.io/badge/Platform-Linux-blue.svg)
![License](https://img.shields.io/badge/License-MIT-green.svg)
![DE](https://img.shields.io/badge/Desktop-GNOME%20%7C%20KDE%20%7C%20Sway%20%7C%20i3-purple.svg)

**OpenCenter** is a high-performance, lightweight, modern Linux control center for **Elgato Key Lights**, **Key Light Air**, **Key Light Mini**, and **Light Strips**. 

Built from the ground up in **Rust** as a modern replacement for unmaintained Linux utilities, OpenCenter provides a native **eframe/egui GUI**, a **DBus System Tray Applet**, **mDNS Zeroconf Auto-Discovery**, **Custom Presets**, **Smooth Transitions**, and a full-featured **CLI Engine**.

---

## 📸 Overview & Features

- 🔍 **Dual Discovery Engine**: Auto-scans local networks for `_elg._tcp.local` via mDNS with an ultra-fast local subnet fallback scan. Manual IP entry support for multi-VLAN or isolated subnets.
- 🖥️ **Desktop GUI Window**: Dark-mode native interface with real-time connection status indicators, individual and master sliders for Brightness (0–100%) and Kelvin Temperature (2900K–7000K).
- 📌 **System Tray Applet**: Native `StatusNotifierItem` DBus applet for seamless top-bar tray integration on GNOME, KDE Plasma, Sway, Wayland, and X11.
- ⚡ **CLI Automation Engine**: Command-line tool designed for scripting, Stream Deck macro keys, and desktop global keyboard shortcuts.
- 🎨 **Scene & Preset Manager**: Includes single-click presets (*Focus 5000K*, *Studio Call 4500K*, *Warm Reading 3000K*, *Night Shift 2700K*, *All Off*) with custom preset creation.
- 🌊 **Smooth Fade Engine**: Gradually interpolates brightness and temperature over configurable millisecond intervals to avoid sudden lighting jumps on camera.
- ⚙️ **Zero-Config Autostart**: Built-in `systemd --user` service script for zero-overhead background operation.

---

## 📱 Supported Devices

- **Elgato Key Light**
- **Elgato Key Light Air**
- **Elgato Key Light Mini**
- **Elgato Key Light MK.2**
- **Elgato Light Strip / Light Strip Pro**

---

## 🛠️ Prerequisites & Build Dependencies

Ensure the following development packages are installed on your Linux system:

#### Ubuntu / Debian / Pop!_OS
```bash
sudo apt update
sudo apt install build-essential pkg-config libdbus-1-dev libx11-dev libxkbcommon-dev
```

#### Fedora / RHEL
```bash
sudo dnf install dbus-devel libX11-devel libxkbcommon-devel
```

#### Arch Linux / Manjaro
```bash
sudo pacman -S base-devel dbus libx11 libxkbcommon
```

---

## ⚙️ Installation

Build and install `opencenter` to `~/.local/bin/` and generate the `systemd` user service:

```bash
chmod +x install.sh
./install.sh
```

### Enable Background Daemon & Tray Applet on Startup

```bash
systemctl --user enable --now opencenter
```

---

## 🚀 Usage Guide

### 1. Graphical User Interface (GUI) & System Tray
Launch the GUI window and system tray applet:
```bash
opencenter gui
```

---

### 2. Command Line Interface (CLI)

#### 🔍 Auto-Discover Devices
```bash
# Scan local network and automatically save discovered devices to config
opencenter discover --save
```

#### 💡 Check Real-Time Device Status
```bash
opencenter status
```

Output example:
```text
💡 Elgato Device Statuses:
  • Key Light Left       | IP: 192.168.1.50    | 🟢 ON  | Brightness:  80% | Temp: 4500K
  • Key Light Right      | IP: 192.168.1.51    | 🟢 ON  | Brightness:  80% | Temp: 4500K
```

#### ⚡ Power Control
```bash
# Toggle power on all lights
opencenter toggle

# Toggle power on a specific light by IP or Name
opencenter toggle 192.168.1.50

# Turn ON or OFF explicitly
opencenter on
opencenter off
```

#### 🎛️ Adjust Brightness & Color Temperature
```bash
# Set brightness to 80% and color temp to 4500K for all lights
opencenter set --brightness 80 --kelvin 4500

# Target a specific device by IP or Name
opencenter set --target 192.168.1.50 --brightness 100 --kelvin 5000
```

#### 🌊 Smooth Transition / Fade
```bash
# Fade to 100% brightness and 5000K over 2000 milliseconds (2 seconds)
opencenter fade --brightness 100 --kelvin 5000 --duration-ms 2000
```

#### 📋 Preset Management
```bash
# List all saved presets
opencenter preset list

# Apply a preset by name
opencenter preset apply "Studio Call"
opencenter preset apply "Warm Reading"
opencenter preset apply "Night Shift"
opencenter preset apply "All Off"

# Save current light state as a new custom preset
opencenter preset save "Streaming Setup"
```

#### ⚡ Flash Light to Identify Location
```bash
opencenter identify 192.168.1.50
```

#### ➕ Manual Device IP Management
```bash
# Add a device manually by IP
opencenter add-ip 192.168.1.55 --name "Desk Key Light"

# Remove a device by IP
opencenter remove-ip 192.168.1.55
```

---

## ⌨️ Desktop Global Hotkey Integration

Bind shell commands directly to keyboard shortcuts in your desktop environment:

### GNOME / KDE / XFCE
Go to **Settings** $\rightarrow$ **Keyboard** $\rightarrow$ **Custom Shortcuts**:

| Action | Command | Recommended Hotkey |
| :--- | :--- | :--- |
| **Toggle Lights Power** | `opencenter toggle` | `Super + Alt + L` |
| **Studio Meeting Preset** | `opencenter preset apply "Studio Call"` | `Super + Alt + S` |
| **Night Shift Mode** | `opencenter preset apply "Night Shift"` | `Super + Alt + N` |
| **Max Brightness** | `opencenter set --brightness 100` | `Super + Alt + Up` |
| **Turn All Off** | `opencenter off` | `Super + Alt + Down` |

### Hyprland (`~/.config/hypr/hyprland.conf`)
```ini
bind = SUPER ALT, L, exec, opencenter toggle
bind = SUPER ALT, S, exec, opencenter preset apply "Studio Call"
bind = SUPER ALT, N, exec, opencenter preset apply "Night Shift"
```

### Sway / i3 (`~/.config/sway/config`)
```ini
bindsym $mod+Mod1+l exec opencenter toggle
bindsym $mod+Mod1+s exec opencenter preset apply "Studio Call"
```

---

## 💾 Configuration Schema

Configuration is automatically stored in `~/.config/opencenter/config.json`:

```json
{
  "devices": [
    {
      "ip": "192.168.1.50",
      "name": "Key Light Left",
      "serial": "AZ123456789",
      "model": "Elgato Key Light Air",
      "enabled": true
    }
  ],
  "groups": [
    {
      "name": "Desk Lights",
      "device_ips": ["192.168.1.50"]
    }
  ],
  "presets": [
    {
      "name": "Focus",
      "on": true,
      "brightness": 80,
      "kelvin": 5000
    },
    {
      "name": "Studio Call",
      "on": true,
      "brightness": 100,
      "kelvin": 4500
    }
  ]
}
```

---

## 📐 Technical Architecture & API Details

Elgato Key Lights communicate locally on **Port 9123** via unauthenticated HTTP JSON:

- **Metadata Endpoint**: `GET http://<ip>:9123/elgato/accessory-info`
- **Status Endpoint**: `GET http://<ip>:9123/elgato/lights`
- **Update Endpoint**: `PUT http://<ip>:9123/elgato/lights`

### Color Temperature Conversion
Elgato devices specify color temperature in **mireds** ($M$):
$$M = \left\lfloor \frac{1,000,000}{\text{Kelvin}} \right\rfloor$$

- **$2900\text{K (Warm Yellow)}$** $\rightarrow 344\text{ mireds}$
- **$7000\text{K (Cool Blue)}$** $\rightarrow 143\text{ mireds}$

OpenCenter handles all mired/kelvin conversions automatically.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE). Inspired by [`monadplus/elgato-keylight`](https://github.com/monadplus/elgato-keylight).
