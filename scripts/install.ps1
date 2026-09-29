[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Archive,
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'Programs\Itemark'),
    [string]$SkillArchive,
    [ValidateSet('codex', 'claude')][string]$SkillProfile,
    [string]$SkillHome
)
$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $Archive -PathType Leaf)) { throw "Archive not found: $Archive" }
if ($SkillArchive -and -not $SkillProfile) { throw '--SkillProfile codex|claude is required with --SkillArchive' }
if ($SkillArchive -and -not (Test-Path -LiteralPath $SkillArchive -PathType Leaf)) { throw "Skill archive not found: $SkillArchive" }
if (-not $SkillArchive -and ($SkillProfile -or $SkillHome)) { throw '--SkillProfile and --SkillHome require --SkillArchive' }

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
    Remove-Item -LiteralPath $tempPath -Recurse -Force
}
