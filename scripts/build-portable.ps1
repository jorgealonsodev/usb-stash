# USB Stash — Build Portable (Windows)
# PowerShell script

Write-Host "USB Stash — Build Portable (Windows)" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

$OutputDir = "dist/usb"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Build GUI and CLI for Windows
Write-Host "[1/3] Compilando GUI y CLI para Windows..." -ForegroundColor Yellow
cargo build --release -p usbstash-gui -p usbstash-cli --target x86_64-pc-windows-msvc

# Copy binaries
Write-Host "[2/3] Copiando binarios Windows..." -ForegroundColor Yellow
Copy-Item "target/x86_64-pc-windows-msvc/release/usbstash-gui.exe" "$OutputDir/usb-stash-gui-win.exe"
Copy-Item "target/x86_64-pc-windows-msvc/release/usbstash.exe" "$OutputDir/usb-stash-win.exe"
Write-Host "       -> $OutputDir/usb-stash-gui-win.exe" -ForegroundColor Green
Write-Host "       -> $OutputDir/usb-stash-win.exe" -ForegroundColor Green

# Copy README
Write-Host "[3/3] Copiando README para el USB..." -ForegroundColor Yellow
Copy-Item "scripts/USB_README.txt" "$OutputDir/README.txt"
Write-Host "       -> $OutputDir/README.txt" -ForegroundColor Green

Write-Host ""
Write-Host "USB Stash listo. Copiá el contenido de $OutputDir a tu USB." -ForegroundColor Green
Write-Host ""
Write-Host "Estructura generada:"
Get-ChildItem $OutputDir | Format-Table Name, Length
