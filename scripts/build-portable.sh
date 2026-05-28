#!/bin/bash
set -euo pipefail

echo "🔐 USB Stash — Build Portable (Linux)"
echo "======================================"
echo ""

OUTPUT_DIR="dist/usb"
mkdir -p "$OUTPUT_DIR"

# Build GUI and CLI for Linux
echo "[1/3] Compilando GUI y CLI para Linux..."
cargo build --release -p usbstash-gui -p usbstash-cli --target x86_64-unknown-linux-gnu

# Copy binaries
echo "[2/3] Copiando binarios Linux..."
cp "target/x86_64-unknown-linux-gnu/release/usbstash-gui" "$OUTPUT_DIR/usb-stash-gui-linux"
cp "target/x86_64-unknown-linux-gnu/release/usbstash" "$OUTPUT_DIR/usb-stash-linux"
chmod +x "$OUTPUT_DIR/usb-stash-gui-linux" "$OUTPUT_DIR/usb-stash-linux"
echo "       → $OUTPUT_DIR/usb-stash-gui-linux"
echo "       → $OUTPUT_DIR/usb-stash-linux"

# Copy README
echo "[3/3] Copiando README para el USB..."
cp scripts/USB_README.txt "$OUTPUT_DIR/README.txt"
echo "       → $OUTPUT_DIR/README.txt"

echo ""
echo "✅ USB Stash listo. Copiá el contenido de $OUTPUT_DIR a tu USB."
echo ""
echo "Estructura generada:"
ls -lh "$OUTPUT_DIR/"
