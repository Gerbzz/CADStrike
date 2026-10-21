<#
.SYNOPSIS
Build or start the CADStrike desktop based on the imported CADCraft foundation.
.DESCRIPTION
Always asks Cargo to use the foundation workspace and its own target directory.
-PlanOnly reports the selection without running Cargo. -BuildOnly builds without
opening a window. The root legacy bracket editor remains a separate explicit command.
#>
[CmdletBinding()]
param(
    [switch]$BuildOnly,
    [switch]$PlanOnly,
    [string[]]$AppArguments = @()
)
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$manifest = Join-Path $repo 'foundation/Cargo.toml'
$target = Join-Path $repo 'foundation/target'
if (-not (Test-Path -LiteralPath $manifest)) { throw "CADCraft foundation manifest not found: $manifest" }
if ($BuildOnly -and $AppArguments.Count) { throw 'Application arguments cannot be used with -BuildOnly.' }

$sourceStamp = 'unversioned'
if ((Test-Path -LiteralPath (Join-Path $repo '.git')) -and (Get-Command git -ErrorAction SilentlyContinue)) {
    $revision = & git -C $repo rev-parse --short=12 HEAD 2>$null
    if ($LASTEXITCODE -eq 0) {
        $sourceStamp = "$revision"
        $changes = & git -C $repo status --porcelain --untracked-files=all
        if ($LASTEXITCODE -ne 0) { throw 'Could not inspect CADStrike source status.' }
        if ($changes) { $sourceStamp += '-dirty' }
    }
}
$action = if ($BuildOnly) { 'build' } else { 'run' }
$cargoArguments = @($action, '--locked', '--manifest-path', $manifest, '--package', 'cadstrike', '--bin', 'cadstrike', '--target-dir', $target)
if (-not $BuildOnly -and $AppArguments.Count) { $cargoArguments += @('--') + $AppArguments }
$plan = [pscustomobject]@{
    Manifest = $manifest
    TargetDirectory = $target
    Package = 'cadstrike'
    Binary = 'cadstrike'
    SourceStamp = $sourceStamp
    DonorCommit = 'c57879a5c8f5e3a6be574a820add5e28eb25e9b1'
    CargoArguments = $cargoArguments
}
if ($PlanOnly) { return $plan }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Cargo is required. See the Rust prerequisites in README.md.' }
Write-Host "CADStrike CADCraft foundation: $sourceStamp"
Write-Host "Manifest: $manifest"
Write-Host "Build output: $target"
$priorStamp = $env:CADCRAFT_BUILD_SHA
try {
    $env:CADCRAFT_BUILD_SHA = $sourceStamp
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) { throw "CADStrike Cargo $action failed with exit code $LASTEXITCODE." }
} finally {
    $env:CADCRAFT_BUILD_SHA = $priorStamp
}
