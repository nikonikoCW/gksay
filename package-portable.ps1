$ErrorActionPreference = "Stop"
$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$releaseExe = Join-Path $projectDir "src-tauri\target\release\gksay.exe"
$portableDir = Join-Path $projectDir "portable\GkSay"

if (-not (Test-Path -LiteralPath $releaseExe)) {
    throw "未找到 release EXE，请先运行 pnpm tauri build --no-bundle"
}

New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
Copy-Item -LiteralPath $releaseExe -Destination (Join-Path $portableDir "GkSay.exe") -Force
Copy-Item -LiteralPath (Join-Path $projectDir "messages.txt") -Destination $portableDir -Force
Copy-Item -LiteralPath (Join-Path $projectDir "config.toml") -Destination $portableDir -Force
Copy-Item -LiteralPath (Join-Path $projectDir "使用说明.txt") -Destination $portableDir -Force

Write-Host "便携目录已生成：$portableDir"
