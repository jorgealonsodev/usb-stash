# USB Stash — Build Portable (Windows)
# PowerShell script

Write-Host "USB Stash — Build Portable (Windows)" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

$OutputDir = "dist/usb"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Build Tauri for Windows
Write-Host "[1/3] Compilando aplicación Tauri para Windows..." -ForegroundColor Yellow
cargo tauri build --target x86_64-pc-windows-msvc

# Copy binary
Write-Host "[2/3] Copiando binario Windows..." -ForegroundColor Yellow
$Exe = Get-ChildItem -Path "src-tauri/target/x86_64-pc-windows-msvc/release" -Filter "usb-stash.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($Exe) {
    Copy-Item $Exe.FullName "$OutputDir/usbstash-win.exe"
    Write-Host "       -> $OutputDir/usbstash-win.exe" -ForegroundColor Green
} else {
    Write-Host "       Binario no encontrado. Ejecutá 'cargo tauri build' primero." -ForegroundColor Red
    exit 1
}

# Copy README
Write-Host "[3/3] Copiando README para el USB..." -ForegroundColor Yellow
Copy-Item "scripts/USB_README.txt" "$OutputDir/README.txt"
Write-Host "       -> $OutputDir/README.txt" -ForegroundColor Green

Write-Host ""
Write-Host "USB Stash listo. Copiá el contenido de $OutputDir a tu USB." -ForegroundColor Green
Write-Host ""
Write-Host "Estructura generada:"
Get-ChildItem $OutputDir | Format-Table Name, Length
