#!/bin/bash
set -euo pipefail

echo "🔐 USB Stash — Build Portable (Linux)"
echo "======================================"
echo ""

OUTPUT_DIR="dist/usb"
mkdir -p "$OUTPUT_DIR"

# Build Tauri for Linux
echo "[1/3] Compilando aplicación Tauri para Linux..."
cargo tauri build --target x86_64-unknown-linux-gnu

# Copy AppImage
echo "[2/3] Copiando binario Linux..."
APPIMAGE=$(ls src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/appimage/*.AppImage 2>/dev/null | head -1)
if [ -n "$APPIMAGE" ]; then
    cp "$APPIMAGE" "$OUTPUT_DIR/usbstash-linux"
    chmod +x "$OUTPUT_DIR/usbstash-linux"
    echo "       → $OUTPUT_DIR/usbstash-linux"
else
    echo "       ⚠ AppImage no encontrado, buscando binario..."
    BIN=$(ls src-tauri/target/x86_64-unknown-linux-gnu/release/usb-stash 2>/dev/null | head -1)
    if [ -n "$BIN" ]; then
        cp "$BIN" "$OUTPUT_DIR/usbstash-linux"
        chmod +x "$OUTPUT_DIR/usbstash-linux"
        echo "       → $OUTPUT_DIR/usbstash-linux"
    else
        echo "       ❌ Binario no encontrado. Ejecutá 'cargo tauri build' primero."
        exit 1
    fi
fi

# Copy README
echo "[3/3] Copiando README para el USB..."
cp scripts/USB_README.txt "$OUTPUT_DIR/README.txt"
echo "       → $OUTPUT_DIR/README.txt"

echo ""
echo "✅ USB Stash listo. Copiá el contenido de $OUTPUT_DIR a tu USB."
echo ""
echo "Estructura generada:"
ls -lh "$OUTPUT_DIR/"
