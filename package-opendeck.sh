#!/usr/bin/env bash
set -e

PLUGIN_DIR="com.akitafuki.opencenter.sdPlugin"
echo "🔨 Building opencenter-opendeck in release mode..."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release -p opencenter-opendeck

echo "📦 Assembling ${PLUGIN_DIR}..."
mkdir -p "${PLUGIN_DIR}/bin"
mkdir -p "${PLUGIN_DIR}/assets"

cp crates/opencenter-opendeck/manifest.json "${PLUGIN_DIR}/"
cp -r crates/opencenter-opendeck/assets/* "${PLUGIN_DIR}/assets/"
cp target/release/opencenter-opendeck "${PLUGIN_DIR}/bin/"

echo "📦 Packaging plugin zip archive..."
rm -f opencenter.zip opencenter-opendeck.zip "${PLUGIN_DIR}.zip" com.akitafuki.opencenter.streamDeckPlugin
zip -r opencenter-opendeck.zip "${PLUGIN_DIR}"

echo "✨ OpenDeck / Stream Deck plugin bundle created at ./${PLUGIN_DIR}"
echo "📦 OpenDeck plugin zip archive created at ./opencenter-opendeck.zip"
echo ""
echo "To install in OpenDeck:"
echo "  Option A (UI Import): In OpenDeck, click 'Install Plugin' or import ./opencenter-opendeck.zip"
echo "  Option B (Manual):    Copy ./${PLUGIN_DIR} into ~/.local/share/OpenDeck/plugins/"
