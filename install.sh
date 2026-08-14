#!/usr/bin/env bash
set -e

echo "🔨 Building OpenCenter (Release mode)..."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release

echo "📦 Installing binary to ~/.local/bin/opencenter..."
mkdir -p ~/.local/bin
cp target/release/opencenter ~/.local/bin/

echo "⚙️ Setting up systemd user service..."
mkdir -p ~/.config/systemd/user
cat << 'EOF' > ~/.config/systemd/user/opencenter.service
[Unit]
Description=OpenCenter - Linux Control Center & System Tray for Elgato Key Lights
After=network.target

[Service]
ExecStart=%h/.local/bin/opencenter gui
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
echo "✅ Installation complete!"
echo "Run 'opencenter --help' for CLI, or start the tray app daemon via:"
echo "  systemctl --user enable --now opencenter"
