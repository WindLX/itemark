# Itemark uninstaller for Windows. Safe to run online:
#   irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/uninstall.ps1 | iex
[CmdletBinding()]
param([string]$Prefix = $(if ($env:ITEMARK_PREFIX) { $env:ITEMARK_PREFIX } else { Join-Path $env:LOCALAPPDATA 'Programs\Itemark' }))
$ErrorActionPreference = 'Stop'

$prefixPath = [IO.Path]::GetFullPath($Prefix)
$binaryPath = Join-Path $prefixPath 'itemark.exe'
$receiptPath = Join-Path $prefixPath '.itemark-install'
if (-not (Test-Path -LiteralPath $receiptPath -PathType Leaf)) { throw "No Itemark installation receipt under $prefixPath" }
$receipt = Get-Content -LiteralPath $receiptPath
if ($receipt -notcontains "prefix=$prefixPath") { throw "Installation receipt does not match $prefixPath" }
if (Test-Path -LiteralPath $binaryPath -PathType Leaf) { Remove-Item -LiteralPath $binaryPath -Force }

$skillEntry = $receipt | Where-Object { $_.StartsWith('skill=') } | Select-Object -First 1
if ($skillEntry) {
    $skillPath = $skillEntry.Substring(6)
    $marker = Join-Path $skillPath '.itemark-managed'
    $skillItem = Get-Item -LiteralPath $skillPath -ErrorAction SilentlyContinue
    if ($skillItem -and ([IO.Path]::GetFileName($skillPath) -ieq 'itemark') -and -not ($skillItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -and (Test-Path -LiteralPath $marker -PathType Leaf) -and ((Get-Content -LiteralPath $marker -Raw).Trim() -eq 'installed by itemark')) {
        Remove-Item -LiteralPath $skillPath -Recurse -Force
    } else {
        Write-Warning "Skill installation marker missing; preserving $skillPath"
    }
}
Remove-Item -LiteralPath $receiptPath -Force
Write-Output "Removed Itemark files installed under $prefixPath; parent directories and project records were preserved."
