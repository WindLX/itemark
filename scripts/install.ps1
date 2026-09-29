# Itemark installer for Windows.
#
# Online install (default, latest release):
#   irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.ps1 | iex
#
# Pin a version or pass options:
#   & ([scriptblock]::Create((irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.ps1))) -Version 0.1.0
#
# Local archive install (offline):
#   .\install.ps1 -Archive .\itemark-v0.1.0-x86_64-pc-windows-msvc.zip
#
# Messages stay ASCII-only so the script also runs under Windows PowerShell 5.1.
[CmdletBinding()]
param(
    [string]$Version = $env:ITEMARK_VERSION,
    [string]$Archive,
    [string]$Prefix = $(if ($env:ITEMARK_PREFIX) { $env:ITEMARK_PREFIX } else { Join-Path $env:LOCALAPPDATA 'Programs\Itemark' }),
    [string]$Repo = $(if ($env:ITEMARK_REPO) { $env:ITEMARK_REPO } else { 'WindLX/itemark' }),
    [string]$BaseUrl = $env:ITEMARK_BASE_URL,
    [string]$SkillArchive,
    [ValidateSet('codex', 'claude')][string]$SkillProfile,
    [string]$SkillHome
)
$ErrorActionPreference = 'Stop'
$releaseTarget = 'x86_64-pc-windows-msvc'

# Windows PowerShell 5.1 can default to TLS 1.0, which GitHub rejects.
if ([Net.ServicePointManager]::SecurityProtocol -notmatch 'Tls12') {
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
}

function Get-ItemarkSha256([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-ItemarkChecksum([string]$FilePath, [string]$SumsPath) {
    $name = [IO.Path]::GetFileName($FilePath)
    $expected = $null
    foreach ($line in Get-Content -LiteralPath $SumsPath) {
        $parts = $line.Trim() -split '\s+', 2
        if ($parts.Count -eq 2 -and $parts[1].Trim() -eq $name) { $expected = $parts[0].ToLowerInvariant(); break }
    }
    if (-not $expected) { throw "SHA256SUMS is missing a checksum for $name" }
    if ((Get-ItemarkSha256 $FilePath) -ne $expected) { throw "Checksum mismatch for $name" }
}

function Save-ItemarkUrl([string]$Url, [string]$Destination) {
    Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
}

$prefixPath = [IO.Path]::GetFullPath($Prefix)
$binaryPath = Join-Path $prefixPath 'itemark.exe'
$receiptPath = Join-Path $prefixPath '.itemark-install'
$tempPath = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString('N'))
$skillDestination = $null
$skillInstalled = $false
$binaryInstalled = $false
$receiptCreated = $false
$skillStage = $null
New-Item -ItemType Directory -Path $tempPath | Out-Null
try {
    if (-not $Archive) {
        if ($BaseUrl -and -not $Version) { throw 'BaseUrl requires an explicit Version' }
        if (-not $Version -or $Version -eq 'latest') {
            $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing -Headers @{ 'User-Agent' = 'itemark-installer' }
            $Version = [string]$release.tag_name
        }
        $Version = $Version.TrimStart('v')
        if ($Version -notmatch '\A[0-9]+\.[0-9]+\.[0-9]+([-+][A-Za-z0-9.]+)?\z') { throw "Invalid version: $Version" }
        if ($env:PROCESSOR_ARCHITECTURE -notin @('AMD64', 'ARM64')) { throw "Unsupported Windows architecture: $env:PROCESSOR_ARCHITECTURE" }
        if (-not $BaseUrl) { $BaseUrl = "https://github.com/$Repo/releases/download/v$Version" }
        Write-Output "Downloading itemark v$Version for $releaseTarget"
        # Keep the published asset names: SHA256SUMS entries are keyed by them.
        $Archive = Join-Path $tempPath "itemark-v$Version-$releaseTarget.zip"
        Save-ItemarkUrl "$BaseUrl/itemark-v$Version-$releaseTarget.zip" $Archive
        $sumsPath = Join-Path $tempPath 'SHA256SUMS'
        Save-ItemarkUrl "$BaseUrl/SHA256SUMS" $sumsPath
        Assert-ItemarkChecksum $Archive $sumsPath
        if ($SkillProfile -and -not $SkillArchive) {
            $SkillArchive = Join-Path $tempPath "itemark-skill-v$Version.zip"
            Save-ItemarkUrl "$BaseUrl/itemark-skill-v$Version.zip" $SkillArchive
            Assert-ItemarkChecksum $SkillArchive $sumsPath
        }
    }

    if (-not (Test-Path -LiteralPath $Archive -PathType Leaf)) { throw "Archive not found: $Archive" }
    if ($SkillArchive -and -not $SkillProfile) { throw '-SkillProfile codex|claude is required with -SkillArchive' }
    if ($SkillArchive -and -not (Test-Path -LiteralPath $SkillArchive -PathType Leaf)) { throw "Skill archive not found: $SkillArchive" }
    if (-not $SkillArchive -and ($SkillProfile -or $SkillHome)) { throw '-SkillProfile and -SkillHome require -SkillArchive' }

    Expand-Archive -LiteralPath $Archive -DestinationPath (Join-Path $tempPath 'bin')
    $sourceBinary = Join-Path (Join-Path $tempPath 'bin') 'itemark.exe'
    if (-not (Test-Path -LiteralPath $sourceBinary -PathType Leaf)) { throw 'Archive must contain itemark.exe at its root' }
    New-Item -ItemType Directory -Force -Path $prefixPath | Out-Null
    if ((Test-Path -LiteralPath $binaryPath) -or (Test-Path -LiteralPath $receiptPath)) { throw "Installation already exists under $prefixPath; run uninstall.ps1 first" }
    Copy-Item -LiteralPath $sourceBinary -Destination $binaryPath
    $binaryInstalled = $true
    $receiptCreated = $true
    @("prefix=$prefixPath") | Set-Content -LiteralPath $receiptPath -Encoding utf8

    if ($SkillArchive) {
        $skillBase = if ($SkillHome) { $SkillHome } elseif ($SkillProfile -eq 'codex') { Join-Path $HOME '.agents\skills' } else { Join-Path $HOME '.claude\skills' }
        $skillBase = [IO.Path]::GetFullPath($skillBase)
        $skillDestination = Join-Path $skillBase 'itemark'
        $skillTemp = Join-Path $tempPath 'skill'
        Expand-Archive -LiteralPath $SkillArchive -DestinationPath $skillTemp
        $skillSource = Join-Path $skillTemp 'itemark'
        if (-not (Test-Path -LiteralPath (Join-Path $skillSource 'SKILL.md') -PathType Leaf)) { throw 'Skill archive must contain itemark/SKILL.md' }
        if (-not (Test-Path -LiteralPath (Join-Path $skillSource 'references\cli.md') -PathType Leaf)) { throw 'Skill archive must contain itemark/references/cli.md' }
        if (Test-Path -LiteralPath $skillDestination) { throw "Skill already exists: $skillDestination" }
        New-Item -ItemType Directory -Force -Path $skillBase | Out-Null
        $skillStage = Join-Path $skillBase ('.itemark-stage-' + [Guid]::NewGuid().ToString('N'))
        Copy-Item -LiteralPath $skillSource -Destination $skillStage -Recurse
        'installed by itemark' | Set-Content -LiteralPath (Join-Path $skillStage '.itemark-managed') -Encoding utf8
        if (Test-Path -LiteralPath $skillDestination) { throw "Skill appeared during install: $skillDestination" }
        Move-Item -LiteralPath $skillStage -Destination $skillDestination
        $skillInstalled = $true
        Add-Content -LiteralPath $receiptPath -Value "skill=$skillDestination" -Encoding utf8
        Write-Output "Installed skill $skillDestination"
    }
    Write-Output "Installed $binaryPath"
    if (($env:PATH -split [IO.Path]::PathSeparator) -notcontains $prefixPath) {
        Write-Output "Add $prefixPath to PATH to run itemark directly."
    }
}
catch {
    if ($skillInstalled -and $skillDestination) {
        Remove-Item -LiteralPath $skillDestination -Recurse -Force
    }
    if ($skillStage -and (Test-Path -LiteralPath $skillStage)) { Remove-Item -LiteralPath $skillStage -Recurse -Force }
    if ($binaryInstalled -and (Test-Path -LiteralPath $binaryPath)) { Remove-Item -LiteralPath $binaryPath -Force }
    if ($receiptCreated -and (Test-Path -LiteralPath $receiptPath)) { Remove-Item -LiteralPath $receiptPath -Force }
    throw
}
finally {
    if (Test-Path -LiteralPath $tempPath) { Remove-Item -LiteralPath $tempPath -Recurse -Force }
}
