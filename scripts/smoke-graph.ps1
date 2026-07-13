<#
.SYNOPSIS
Runs the Relascope graph/import/incremental smoke test end-to-end.

.DESCRIPTION
Builds the Relascope CLI once, creates a temporary workspace, copies the checked-in
polyrepo fixture to a temporary mutable fixture directory, registers the copied repos,
verifies graph/import materialization, verifies unchanged scan idempotency, then mutates
and removes files in the copied fixture to verify incremental modified/removed counts
and stale import behavior.

This script is intentionally local-only. It does not use network, AI, or execute code
from registered repositories.

.EXAMPLE
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1

.EXAMPLE
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1 -KeepWorkspace
#>

param(
    [string]$Workspace = "",
    [switch]$KeepWorkspace
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Assert-Contains {
    param(
        [string]$Text,
        [string]$Expected,
        [string]$Context
    )

    if (-not $Text.Contains($Expected)) {
        Write-Host "`n--- Output ---" -ForegroundColor Yellow
        Write-Host $Text
        Write-Host "--- End output ---`n" -ForegroundColor Yellow
        throw "Expected '$Expected' in $Context"
    }
}

function Assert-NotContains {
    param(
        [string]$Text,
        [string]$Unexpected,
        [string]$Context
    )

    if ($Text.Contains($Unexpected)) {
        Write-Host "`n--- Output ---" -ForegroundColor Yellow
        Write-Host $Text
        Write-Host "--- End output ---`n" -ForegroundColor Yellow
        throw "Did not expect '$Unexpected' in $Context"
    }
}

function Invoke-Relascope {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$RelascopeArgs)

    Write-Host "relascope $($RelascopeArgs -join ' ')" -ForegroundColor DarkGray
    $output = & $RelascopeExe @RelascopeArgs 2>&1
    $text = ($output | Out-String).Trim()

    if ($LASTEXITCODE -ne 0) {
        Write-Host $text
        throw "relascope command failed: $($RelascopeArgs -join ' ')"
    }

    return $text
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
$Manifest = Join-Path $RepoRoot "Cargo.toml"
$FixtureRoot = Join-Path $RepoRoot "fixtures\polyrepo-basic"
$RelascopeExe = Join-Path $RepoRoot "target\debug\relascope.exe"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found in PATH. Install Rust with rustup before running this smoke test."
}

if (-not (Test-Path $Manifest)) {
    throw "Cargo.toml not found at $Manifest"
}

if (-not (Test-Path $FixtureRoot)) {
    throw "Fixture root not found at $FixtureRoot"
}

Write-Step "Building Relascope CLI"
& cargo build --manifest-path $Manifest -p relascope-cli
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed"
}
if (-not (Test-Path $RelascopeExe)) {
    throw "Relascope executable not found at $RelascopeExe"
}

if ([string]::IsNullOrWhiteSpace($Workspace)) {
    $Workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("relascope-graph-smoke-" + [Guid]::NewGuid().ToString("N"))
}

$Workspace = [System.IO.Path]::GetFullPath($Workspace)
$MutableFixtureRoot = Join-Path $Workspace "fixtures\polyrepo-basic"

Write-Step "Preparing smoke workspace"
if (Test-Path $Workspace) {
    Remove-Item -Recurse -Force $Workspace
}
New-Item -ItemType Directory -Path $Workspace | Out-Null
New-Item -ItemType Directory -Path (Split-Path -Parent $MutableFixtureRoot) | Out-Null
Copy-Item -Recurse -Force $FixtureRoot $MutableFixtureRoot
Write-Host "workspace: $Workspace"
Write-Host "mutable fixture: $MutableFixtureRoot"

try {
    Write-Step "Initializing Relascope workspace"
    $init = Invoke-Relascope "init" $Workspace
    Assert-Contains $init "Initialized Relascope workspace" "init output"
    Write-Host $init

    Push-Location $Workspace

    Write-Step "Registering copied fixture repositories"
    $api = Join-Path $MutableFixtureRoot "api-python"
    $web = Join-Path $MutableFixtureRoot "web-typescript"
    $infra = Join-Path $MutableFixtureRoot "infra-config"

    $addApi = Invoke-Relascope "repo" "add" $api "--id" "api"
    $addWeb = Invoke-Relascope "repo" "add" $web "--id" "web"
    $addInfra = Invoke-Relascope "repo" "add" $infra "--id" "infra"
    Assert-Contains $addApi "id: api" "repo add api output"
    Assert-Contains $addWeb "id: web" "repo add web output"
    Assert-Contains $addInfra "id: infra" "repo add infra output"

    Write-Step "Listing registered repositories"
    $repoList = Invoke-Relascope "repo" "list"
    Assert-Contains $repoList "api" "repo list output"
    Assert-Contains $repoList "web" "repo list output"
    Assert-Contains $repoList "infra" "repo list output"
    Write-Host $repoList

    Write-Step "Checking doctor before scan"
    $doctorBefore = Invoke-Relascope "doctor"
    Assert-Contains $doctorBefore "Relascope doctor" "doctor before scan"
    Assert-Contains $doctorBefore "repositories: 3 total, 3 available, 0 unavailable" "doctor before scan"
    Write-Host $doctorBefore

    Write-Step "Checking graph summary before scan"
    $before = Invoke-Relascope "graph" "summary"
    Assert-Contains $before "no graph has been materialized yet" "graph summary before scan"
    Write-Host $before

    Write-Step "Checking graph imports before scan"
    $importsBefore = Invoke-Relascope "graph" "imports"
    Assert-Contains $importsBefore "no imports have been materialized yet" "graph imports before scan"
    Write-Host $importsBefore

    Write-Step "Running first scan"
    $scanOne = Invoke-Relascope "scan"
    Assert-Contains $scanOne "Scanning repositories..." "first scan output"
    Assert-Contains $scanOne "Scan completed" "first scan output"
    Assert-Contains $scanOne "files: 12" "first scan output"
    Assert-Contains $scanOne "added: 12" "first scan output"
    Assert-Contains $scanOne "modified: 0" "first scan output"
    Assert-Contains $scanOne "removed: 0" "first scan output"
    Write-Host $scanOne

    Write-Step "Checking graph summary after first scan"
    $summaryOne = Invoke-Relascope "graph" "summary"
    Assert-Contains $summaryOne "nodes: 21" "graph summary after first scan"
    Assert-Contains $summaryOne "File: 12" "graph summary after first scan"
    Assert-Contains $summaryOne "Module: 5" "graph summary after first scan"
    Assert-Contains $summaryOne "Repository: 3" "graph summary after first scan"
    Assert-Contains $summaryOne "Workspace: 1" "graph summary after first scan"
    Assert-Contains $summaryOne "edges: 22" "graph summary after first scan"
    Assert-Contains $summaryOne "CONTAINS: 19" "graph summary after first scan"
    Assert-Contains $summaryOne "IMPORTS: 3" "graph summary after first scan"
    Write-Host $summaryOne

    Write-Step "Checking graph summary JSON after first scan"
    $summaryJson = Invoke-Relascope "graph" "summary" "--format" "json"
    Assert-Contains $summaryJson '"total_nodes": 21' "graph summary json"
    Assert-Contains $summaryJson '"IMPORTS": 3' "graph summary json"
    Write-Host $summaryJson

    Write-Step "Checking doctor after first scan"
    $doctorAfter = Invoke-Relascope "doctor"
    Assert-Contains $doctorAfter "last_scan: completed" "doctor after scan"
    Assert-Contains $doctorAfter "unverified_imports: 1" "doctor after scan"
    Write-Host $doctorAfter

    Write-Step "Checking graph imports after first scan"
    $importsOne = Invoke-Relascope "graph" "imports"
    Assert-Contains $importsOne "api/app.py -> api/settings.py [active]" "graph imports after first scan"
    Assert-Contains $importsOne "api/app.py -> api:os [unverified]" "graph imports after first scan"
    Assert-Contains $importsOne "web/src/main.ts -> web/src/message.ts [active]" "graph imports after first scan"
    Assert-Contains $importsOne "evidence: app.py:1" "graph imports after first scan"
    Assert-Contains $importsOne "evidence: src/main.ts:1" "graph imports after first scan"
    Write-Host $importsOne

    Write-Step "Checking graph imports JSON after first scan"
    $importsJson = Invoke-Relascope "graph" "imports" "--format" "json"
    Assert-Contains $importsJson '"source": "api/app.py"' "graph imports json"
    Assert-Contains $importsJson '"status": "unverified"' "graph imports json"
    Write-Host $importsJson

    Write-Step "Running unchanged second scan"
    $scanTwo = Invoke-Relascope "scan" "--silent"
    Assert-Contains $scanTwo "files: 12" "second scan output"
    Assert-Contains $scanTwo "added: 0" "second scan output"
    Assert-Contains $scanTwo "modified: 0" "second scan output"
    Assert-Contains $scanTwo "removed: 0" "second scan output"
    Write-Host $scanTwo

    Write-Step "Checking scan JSON on unchanged state"
    $scanJson = Invoke-Relascope "scan" "--format" "json"
    Assert-Contains $scanJson '"status": "completed"' "scan json"
    Assert-Contains $scanJson '"added": 0' "scan json"
    Assert-Contains $scanJson '"modified": 0' "scan json"
    Assert-Contains $scanJson '"removed": 0' "scan json"
    Write-Host $scanJson

    Write-Step "Removing one import line from copied api app.py"
    $apiApp = Join-Path $api "app.py"
    @'
from .settings import CONFIG


def hello() -> str:
    return CONFIG.get("message", "hello from api")
'@ | Set-Content -NoNewline -Path $apiApp

    Write-Step "Running scan after modifying copied api app.py"
    $scanThree = Invoke-Relascope "scan" "--silent"
    Assert-Contains $scanThree "files: 12" "third scan output"
    Assert-Contains $scanThree "added: 0" "third scan output"
    Assert-Contains $scanThree "modified: 1" "third scan output"
    Assert-Contains $scanThree "removed: 0" "third scan output"
    Write-Host $scanThree

    Write-Step "Checking stale import after modification"
    $importsAfterModification = Invoke-Relascope "graph" "imports"
    Assert-Contains $importsAfterModification "api/app.py -> api:os [stale]" "graph imports after modification"
    Assert-Contains $importsAfterModification "api/app.py -> api/settings.py [active]" "graph imports after modification"
    Write-Host $importsAfterModification

    Write-Step "Removing copied api settings.py"
    Remove-Item -Force (Join-Path $api "settings.py")

    Write-Step "Running scan after removing copied api settings.py"
    $scanFour = Invoke-Relascope "scan" "--silent"
    Assert-Contains $scanFour "files: 11" "fourth scan output"
    Assert-Contains $scanFour "added: 0" "fourth scan output"
    Assert-Contains $scanFour "modified: 0" "fourth scan output"
    Assert-Contains $scanFour "removed: 1" "fourth scan output"
    Write-Host $scanFour

    Write-Step "Checking stale/resolved import behavior after removal"
    $importsAfterRemoval = Invoke-Relascope "graph" "imports"
    Assert-Contains $importsAfterRemoval "api/app.py -> api/settings.py [stale]" "graph imports after removal"
    Assert-Contains $importsAfterRemoval "api/app.py -> api:.settings [unverified]" "graph imports after removal"
    Write-Host $importsAfterRemoval

    Write-Step "Removing infra repository registration"
    $removeInfra = Invoke-Relascope "repo" "remove" "infra"
    Assert-Contains $removeInfra "Removed repository registration" "repo remove infra"
    if (-not (Test-Path $infra)) {
        throw "repo remove deleted files from disk: $infra"
    }
    $repoListAfterRemove = Invoke-Relascope "repo" "list"
    Assert-NotContains $repoListAfterRemove "  - infra" "repo list after remove"
    Write-Host $repoListAfterRemove

    Write-Step "Smoke test passed"
    Write-Host "Relascope graph/import/incremental smoke test passed." -ForegroundColor Green
}
finally {
    Pop-Location -ErrorAction SilentlyContinue

    if ($KeepWorkspace) {
        Write-Host "Keeping smoke workspace: $Workspace" -ForegroundColor Yellow
    }
    else {
        if (Test-Path $Workspace) {
            Remove-Item -Recurse -Force $Workspace
        }
    }
}
