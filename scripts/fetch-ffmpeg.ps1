# Downloads the BtbN GPL win64 FFmpeg build and places ffmpeg.exe / ffprobe.exe in src-tauri/binaries
# as Tauri "external binaries" (target-triple suffix). Re-run safely; skips if already present.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root 'src-tauri\binaries'
$triple = 'x86_64-pc-windows-msvc'
if ((Test-Path "$dest\ffmpeg-$triple.exe") -and (Test-Path "$dest\ffprobe-$triple.exe")) { Write-Host 'FFmpeg already present.'; exit 0 }
New-Item -ItemType Directory -Force $dest | Out-Null
$url = 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip'
$tmp = Join-Path $env:TEMP 'rimlight-ffmpeg'
Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $tmp | Out-Null
$zip = Join-Path $tmp 'ffmpeg.zip'
Write-Host "Downloading $url"
Invoke-WebRequest $url -OutFile $zip -UseBasicParsing
Expand-Archive $zip -DestinationPath $tmp -Force
$bin = Get-ChildItem $tmp -Recurse -Filter ffmpeg.exe | Select-Object -First 1
Copy-Item $bin.FullName "$dest\ffmpeg-$triple.exe" -Force
Copy-Item (Join-Path $bin.DirectoryName 'ffprobe.exe') "$dest\ffprobe-$triple.exe" -Force
Remove-Item -Recurse -Force $tmp
Write-Host 'Done.'
