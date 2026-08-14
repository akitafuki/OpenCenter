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
cp target/release/opencenter-opendeck "${PLUGIN_DIR}/bin/"

echo "✨ OpenDeck / Stream Deck plugin bundle created at ./${PLUGIN_DIR}"
echo ""
echo "To install in OpenDeck:"
echo "  Copy ./${PLUGIN_DIR} into your OpenDeck plugins folder (e.g. ~/.local/share/OpenDeck/plugins/)"
