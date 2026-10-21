# Lightweight launcher contract checks. Cargo is intercepted; no app or build is run.
$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '../scripts/Start-CADStrike.ps1'
if (-not (Test-Path -LiteralPath $launcher)) { throw 'Foundation launcher is missing' }
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$manifest = Join-Path $repo 'foundation/Cargo.toml'
$target = Join-Path $repo 'foundation/target'
$global:cadstrikeLauncherCalls = @()
$global:cadstrikeLauncherStamps = @()
$global:cadstrikeResourceRequirements = @()
function cargo {
    $global:cadstrikeLauncherCalls += ,@($args)
    $global:cadstrikeLauncherStamps += $env:CADCRAFT_BUILD_SHA
    $global:cadstrikeResourceRequirements += $env:CADCRAFT_REQUIRE_WINRES
    Set-Variable -Name LASTEXITCODE -Value $global:cadstrikeMockExit -Scope 1
}
function Assert-Equal($actual, $expected, $message) {
    if ($actual -cne $expected) { throw "$message. Expected '$expected', got '$actual'" }
}
$global:cadstrikeMockExit = 0
$priorStamp = $env:CADCRAFT_BUILD_SHA
$priorWinres = $env:CADCRAFT_REQUIRE_WINRES
$env:CADCRAFT_BUILD_SHA = 'test-prior-value'
$env:CADCRAFT_REQUIRE_WINRES = 'test-prior-winres'
try {
    $plan = & $launcher -PlanOnly -AppArguments @('--sample', "drawing with 'quote'.dxf")
    Assert-Equal $cadstrikeLauncherCalls.Count 0 'PlanOnly must not invoke Cargo'
    Assert-Equal $plan.Manifest $manifest 'Launcher must select the foundation manifest'
    Assert-Equal $plan.TargetDirectory $target 'Launcher must separate foundation and legacy output'
    Assert-Equal $plan.Package 'cadstrike' 'Launcher must select the donor desktop package'
    Assert-Equal $plan.Binary 'cadstrike' 'Launcher must select the donor desktop binary'
    Assert-Equal $plan.DonorCommit 'c57879a5c8f5e3a6be574a820add5e28eb25e9b1' 'Plan must report the imported donor pin'
    Assert-Equal $env:CADCRAFT_BUILD_SHA 'test-prior-value' 'PlanOnly must preserve the caller environment'
    Assert-Equal $env:CADCRAFT_REQUIRE_WINRES 'test-prior-winres' 'PlanOnly must preserve the caller resource requirement'

    & $launcher -AppArguments @('--sample', "drawing with 'quote'.dxf")
    $expected = @('run', '--locked', '--manifest-path', $manifest, '--package', 'cadstrike', '--bin', 'cadstrike', '--target-dir', $target, '--', '--sample', "drawing with 'quote'.dxf")
    Assert-Equal ($cadstrikeLauncherCalls[0] | ConvertTo-Json -Compress) ($expected | ConvertTo-Json -Compress) 'Run must select the foundation and preserve argument boundaries'
    Assert-Equal $cadstrikeLauncherStamps[0] $plan.SourceStamp 'Run must pass the inspected source identity to the existing build hook'
    Assert-Equal $cadstrikeResourceRequirements[0] '1' 'Launcher builds must require embedded Windows identity resources'
    Assert-Equal $env:CADCRAFT_BUILD_SHA 'test-prior-value' 'Run must restore the caller environment'
    Assert-Equal $env:CADCRAFT_REQUIRE_WINRES 'test-prior-winres' 'Run must restore the caller resource requirement'

    & $launcher -BuildOnly
    Assert-Equal $cadstrikeLauncherCalls[1][0] 'build' 'BuildOnly must build, without running the app'
    Assert-Equal $cadstrikeLauncherCalls[1].Count 10 'BuildOnly must not append application arguments'

    $failed = $false
    $global:cadstrikeMockExit = 42
    try { & $launcher -BuildOnly } catch { $failed = $_.Exception.Message -match '42' }
    Assert-Equal $failed $true 'Cargo failure must be reported'
    Assert-Equal $env:CADCRAFT_BUILD_SHA 'test-prior-value' 'Failure must restore the caller environment'
    Assert-Equal $env:CADCRAFT_REQUIRE_WINRES 'test-prior-winres' 'Failure must restore the caller resource requirement'
    Assert-Equal $cadstrikeLauncherCalls.Count 3 'Exactly three intercepted Cargo calls expected'
    'PASS: foundation selection, isolated output, argument forwarding, plan/build behavior, failure handling, environment restoration'
} finally {
    $env:CADCRAFT_BUILD_SHA = $priorStamp
    $env:CADCRAFT_REQUIRE_WINRES = $priorWinres
}
