$ErrorActionPreference = "Stop"

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = "sewandev/Reverbic"

# Solo agrega parametros de proxy si Windows tiene uno configurado para esta URL;
# pasar -ProxyUseDefaultCredentials sin -Proxy provoca un error en redes sin proxy.
function Get-ProxyParams([string]$uri) {
    $proxy = [System.Net.WebRequest]::GetSystemWebProxy().GetProxy([Uri]$uri)
    if ($proxy -and $proxy.AbsoluteUri -ne $uri) {
        return @{ Proxy = $proxy; ProxyUseDefaultCredentials = $true }
    }
    return @{}
}

Write-Host ""
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host "        Instalador de Reverbic         " -ForegroundColor Cyan
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host ""

# REVERBIC_PRERELEASE=1 permite probar versiones beta publicadas como pre-release en GitHub.
if ($env:REVERBIC_PRERELEASE) {
    $releaseUrl = "https://api.github.com/repos/$repo/releases"
} else {
    $releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
}

Write-Host "[1/4] Buscando la ultima version en GitHub..."
try {
    $proxyParams = Get-ProxyParams $releaseUrl
    $response = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing @proxyParams
} catch {
    if ($_.Exception.Response -and $_.Exception.Response.StatusCode.value__ -eq 403) {
        Write-Host ""
        Write-Host "GitHub limito temporalmente las peticiones anonimas desde tu red (HTTP 403)." -ForegroundColor Red
        Write-Host "Intenta de nuevo en unos minutos o descarga Reverbic manualmente desde:" -ForegroundColor Yellow
        Write-Host "  https://github.com/$repo/releases/latest" -ForegroundColor Cyan
    } else {
        Write-Host ""
        Write-Host "No se pudo contactar a GitHub: $($_.Exception.Message)" -ForegroundColor Red
    }
    exit 1
}

$releaseData = if ($env:REVERBIC_PRERELEASE) { $response | Select-Object -First 1 } else { $response }

# Solo se publican binarios x86_64. En ARM64 se usa ese mismo binario via emulacion.
switch ($env:PROCESSOR_ARCHITECTURE) {
    "ARM64" { $patterns = @("aarch64-windows", "x86_64-windows") }
    "AMD64" { $patterns = @("x86_64-windows") }
    default {
        Write-Host ""
        Write-Host "Arquitectura no soportada: $($env:PROCESSOR_ARCHITECTURE)." -ForegroundColor Red
        Write-Host "Reverbic requiere Windows de 64 bits (x86_64 o ARM64)." -ForegroundColor Yellow
        exit 1
    }
}

$asset = $null
foreach ($pattern in $patterns) {
    $asset = $releaseData.assets | Where-Object { $_.name -match "$pattern\.exe$" } | Select-Object -First 1
    if ($asset) { break }
}

if (-not $asset) {
    Write-Host ""
    Write-Host "Error: No se encontro un ejecutable de Windows compatible en el release mas reciente." -ForegroundColor Red
    exit 1
}

$downloadUrl = $asset.browser_download_url
$fileName = $asset.name
$tempDir = [System.IO.Path]::GetTempPath()
$tempPath = Join-Path $tempDir $fileName

Write-Host "[2/4] Descargando $($releaseData.tag_name) ($fileName)..."
try {
    $proxyParams = Get-ProxyParams $downloadUrl
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath -UseBasicParsing @proxyParams
} catch {
    Write-Host ""
    Write-Host "La descarga fallo: $($_.Exception.Message)" -ForegroundColor Red
    Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "[3/4] Verificando integridad..."
if ($asset.digest -and $asset.digest -match '^sha256:([0-9a-fA-F]{64})$') {
    $expectedHash = $matches[1].ToLower()
    $actualHash = (Get-FileHash -Path $tempPath -Algorithm SHA256).Hash.ToLower()
    if ($actualHash -ne $expectedHash) {
        Write-Host ""
        Write-Host "La verificacion de integridad SHA256 fallo." -ForegroundColor Red
        Write-Host "La descarga pudo haberse corrompido o manipulado. Instalacion abortada." -ForegroundColor Yellow
        Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
        exit 1
    }
    Write-Host "      Hash SHA256 verificado correctamente." -ForegroundColor DarkGray
} else {
    Write-Host "      Advertencia: el release no incluye un hash SHA256 para verificar." -ForegroundColor DarkYellow
}

# Quita la marca "descargado de internet" del binario ya verificado para evitar
# el aviso de SmartScreen al iniciarlo.
Unblock-File -Path $tempPath -ErrorAction SilentlyContinue

Write-Host "[4/4] Iniciando Reverbic..."
Write-Host ""
Write-Host "Reverbic se abrira a continuacion en esta misma terminal." -ForegroundColor Yellow
Write-Host "Presiona 'q' para salir y ver el resumen de la instalacion." -ForegroundColor Yellow
Write-Host ""

& $tempPath

Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue

# Solo se agrega la carpeta de instalacion al PATH de esta sesion, sin pisar
# otras variables que la sesion actual pudiera tener (ej. entornos virtuales).
$installDir = Join-Path $env:LOCALAPPDATA "Programs\reverbic"
if ($env:PATH -notlike "*$installDir*") {
    $env:PATH += ";$installDir"
}

Write-Host ""
Write-Host "======================================================" -ForegroundColor Green
Write-Host "Instalacion completada con exito!" -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Reverbic se ha copiado a tu carpeta local y se ha anadido a tu PATH." -ForegroundColor Yellow
Write-Host "Ya puedes escribir 'reverbic' desde esta o cualquier otra terminal." -ForegroundColor Yellow
Write-Host ""
