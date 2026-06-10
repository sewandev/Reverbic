$ErrorActionPreference = "Stop"

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = "sewandev/Reverbic"

# Only add proxy parameters if Windows has one configured for this URL;
# passing -ProxyUseDefaultCredentials without -Proxy causes an error on networks without a proxy.
function Get-ProxyParams([string]$uri) {
    $proxy = [System.Net.WebRequest]::GetSystemWebProxy().GetProxy([Uri]$uri)
    if ($proxy -and $proxy.AbsoluteUri -ne $uri) {
        return @{ Proxy = $proxy; ProxyUseDefaultCredentials = $true }
    }
    return @{}
}

Write-Host ""
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host "          Reverbic Installer           " -ForegroundColor Cyan
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host ""

# Any non-empty value in REVERBIC_PRERELEASE enables this mode
# (includes versions marked as pre-release on GitHub).
if ($env:REVERBIC_PRERELEASE) {
    $releaseUrl = "https://api.github.com/repos/$repo/releases"
} else {
    $releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
}

Write-Host "[1/4] Looking for the latest version on GitHub..."
try {
    $proxyParams = Get-ProxyParams $releaseUrl
    $response = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing @proxyParams
} catch {
    if ($_.Exception.Response -and $_.Exception.Response.StatusCode.value__ -eq 403) {
        Write-Host ""
        Write-Host "GitHub temporarily rate-limited anonymous requests from your network (HTTP 403)." -ForegroundColor Red
        Write-Host "Try again in a few minutes or download Reverbic manually from:" -ForegroundColor Yellow
        Write-Host "  https://github.com/$repo/releases/latest" -ForegroundColor Cyan
    } else {
        Write-Host ""
        Write-Host "Could not contact GitHub: $($_.Exception.Message)" -ForegroundColor Red
    }
    exit 1
}

$releaseData = if ($env:REVERBIC_PRERELEASE) { $response | Select-Object -First 1 } else { $response }

# PROCESSOR_ARCHITECTURE reflects the process architecture, not the system's:
# on 32-bit PowerShell running on 64-bit Windows, the real one is in PROCESSOR_ARCHITEW6432.
$architecture = $env:PROCESSOR_ARCHITECTURE
if ($env:PROCESSOR_ARCHITEW6432) {
    $architecture = $env:PROCESSOR_ARCHITEW6432
}

# Only x86_64 binaries are published. On ARM64, the same binary is used via emulation.
switch ($architecture) {
    "ARM64" { $patterns = @("aarch64-windows", "x86_64-windows") }
    "AMD64" { $patterns = @("x86_64-windows") }
    default {
        Write-Host ""
        Write-Host "Unsupported architecture: $architecture." -ForegroundColor Red
        Write-Host "Reverbic requires 64-bit Windows (x86_64 or ARM64)." -ForegroundColor Yellow
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
    Write-Host "Error: No release with a compatible Windows executable was found." -ForegroundColor Red
    exit 1
}

$downloadUrl = $asset.browser_download_url
$fileName = $asset.name
$tempDir = [System.IO.Path]::GetTempPath()
$tempPath = Join-Path $tempDir $fileName

Write-Host "[2/4] Downloading $($releaseData.tag_name) ($fileName)..."
try {
    $proxyParams = Get-ProxyParams $downloadUrl
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath -UseBasicParsing @proxyParams
} catch {
    Write-Host ""
    Write-Host "Download failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception -is [System.IO.IOException]) {
        Write-Host "If Reverbic is already open from a previous installation, close it (press 'q') and try again." -ForegroundColor Yellow
    }
    Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "[3/4] Verifying integrity..."
try {
    if ($asset.digest -and $asset.digest -match '^sha256:([0-9a-fA-F]{64})$') {
        $expectedHash = $matches[1].ToLower()
        $actualHash = (Get-FileHash -Path $tempPath -Algorithm SHA256).Hash.ToLower()
        if ($actualHash -ne $expectedHash) {
            Write-Host ""
            Write-Host "SHA256 integrity verification failed." -ForegroundColor Red
            Write-Host "The download may have been corrupted or tampered with. Installation aborted." -ForegroundColor Yellow
            Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
            exit 1
        }
        Write-Host "      SHA256 hash verified successfully." -ForegroundColor DarkGray
    } else {
        Write-Host "      Warning: the release does not include a SHA256 hash to verify." -ForegroundColor DarkYellow
    }
} catch {
    Write-Host ""
    Write-Host "Could not verify the downloaded file: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Your antivirus may have quarantined or is blocking it." -ForegroundColor Yellow
    Write-Host "Check Windows Defender or your antivirus and try again." -ForegroundColor Yellow
    Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
    exit 1
}

# Removes the "downloaded from the internet" mark from the already-verified binary to avoid
# the SmartScreen warning when launching it.
Unblock-File -Path $tempPath -ErrorAction SilentlyContinue

Write-Host "[4/4] Starting Reverbic..."
Write-Host ""
Write-Host "Reverbic will now open in this same terminal." -ForegroundColor Yellow
Write-Host "Press 'q' to exit and see the installation summary." -ForegroundColor Yellow
Write-Host ""

try {
    & $tempPath
} catch {
    Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
    Write-Host ""
    Write-Host "Could not start Reverbic: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Your antivirus may have blocked the executable. Check Windows Defender" -ForegroundColor Yellow
    Write-Host "or download it manually from: https://github.com/$repo/releases/latest" -ForegroundColor Yellow
    exit 1
}

Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue

# Only append Reverbic's install folder to this session's PATH, without overwriting
# other variables the current session might have (e.g. virtual environments).
if ($env:LOCALAPPDATA) {
    $installDir = Join-Path $env:LOCALAPPDATA "Programs\reverbic"
    if ($env:PATH -notlike "*$installDir*") {
        $env:PATH += ";$installDir"
    }
}

Write-Host ""
Write-Host "======================================================" -ForegroundColor Green
Write-Host "Installation completed successfully!" -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Reverbic has been copied to your local folder and added to your PATH." -ForegroundColor Yellow
Write-Host "You can now type 'reverbic' from this or any other terminal." -ForegroundColor Yellow
Write-Host ""
