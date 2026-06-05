$ErrorActionPreference = "Stop"

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

Write-Host ""
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host "        Instalador de Reverbic         " -ForegroundColor Cyan
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "[1/3] Buscando la ultima version en GitHub..."
$releaseUrl = "https://api.github.com/repos/sewandev/Reverbic/releases/latest"
$releaseData = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing

$asset = $releaseData.assets | Where-Object { $_.name -match "x86_64-windows\.exe$" } | Select-Object -First 1

if (-not $asset) {
    Write-Error "Error: No se encontro el ejecutable de Windows en el release mas reciente."
    exit 1
}

$downloadUrl = $asset.browser_download_url
$fileName = $asset.name
$tempDir = [System.IO.Path]::GetTempPath()
$tempPath = Join-Path $tempDir $fileName

Write-Host "[2/3] Descargando $($releaseData.tag_name) ($fileName)..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath -UseBasicParsing

Write-Host "[3/3] Ejecutando Reverbic para autoinstalacion..."
# Ejecutamos el binario para que haga su propia magia (copiarse a AppData y agregarse al PATH)
Start-Process -FilePath $tempPath -Wait

Write-Host ""
Write-Host "======================================================" -ForegroundColor Green
Write-Host "¡Instalacion completada con exito!" -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Reverbic se ha copiado a tu carpeta local y se ha anadido a tu PATH." -ForegroundColor Yellow
Write-Host "Por favor, CIERRA ESTA TERMINAL, abre una nueva y escribe 'reverbic' para comenzar." -ForegroundColor Yellow
Write-Host ""
