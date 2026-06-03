#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo    = 'sewandev/Reverbic'
$appName = 'reverbic'
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\reverbic'

Write-Host "Obteniendo ultima version..."
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
$asset   = $release.assets | Where-Object { $_.name -like '*.exe' } | Select-Object -First 1

if (-not $asset) {
    Write-Error "No se encontro el binario en la ultima release."
    exit 1
}

$version = $release.tag_name
$url     = $asset.browser_download_url
$exePath = Join-Path $installDir "$appName.exe"

Write-Host "Instalando $appName $version..."

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

Invoke-WebRequest -Uri $url -OutFile $exePath -UseBasicParsing
Write-Host "Binario instalado en: $exePath"

$userPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable('PATH', "$userPath;$installDir", 'User')
    Write-Host "Carpeta agregada al PATH del usuario."
    Write-Host "Reinicia la terminal y ejecuta: $appName"
} else {
    Write-Host "Carpeta ya estaba en PATH. Ejecuta: $appName"
}
