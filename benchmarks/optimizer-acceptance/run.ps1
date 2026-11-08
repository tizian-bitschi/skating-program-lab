param(
    [Parameter(Mandatory)][string]$ExpectedRevision,
    [string]$Baseline = 'cf1343fbf82829ca10969562a71b3b48cd0bbcf8',
    [ValidateRange(3, 20)][int]$Runs = 5,
    [switch]$VerifyOnly
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
function Git([string[]]$Arguments) {
    $value = & git -C $root @Arguments
    if ($LASTEXITCODE -ne 0) { throw "git failed: $Arguments" }
    return $value
}
$revision = Git @('rev-parse', '--verify', "$ExpectedRevision^{commit}")
if ((Git @('rev-parse', 'HEAD')) -ne $revision) { throw 'Expected revision differs from HEAD' }
if (Git @('status', '--porcelain', '--untracked-files=normal')) { throw 'Requires a clean tracked/untracked source checkout' }
$before = Git @('rev-parse', '--verify', "$Baseline^{commit}")
# These are the complete shared fixture/generation dependencies, not just the TSVs.
$inputs = @('tests/support/workflows.rs', 'tests/support/generation_defaults.rs', 'tests/support/senior_benchmarks.rs', 'tests/fixtures/grand-prix-2026-27', 'rust-toolchain.toml', 'Cargo.toml', 'benchmarks/senior-grand-prix/Cargo.lock')
foreach ($path in $inputs) {
    if ((Git @('rev-parse', "${before}:$path")) -ne (Git @('rev-parse', "${revision}:$path"))) { throw "Input differs: $path" }
}
if ($VerifyOnly) { "Verified revisions and identical input trees: $before / $revision"; return }
$output = Join-Path $root "target/acceptance-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $output | Out-Null
$overlay = @('src/optimizer/proposal_diagnostics.rs', 'src/optimizer/proposal_diagnostics/acceptance.rs')
@("before=$before", "after=$revision", "runs=$Runs", 'budget_ms=250; proposals=200; ties=100 (Senior ties=1000); default seed=42',
  'mode=debug; Pumpkin=0.5.0; compilation excluded; test-only identical instrumentation overlay',
  "os=$([Environment]::OSVersion.VersionString)", "processor=$env:PROCESSOR_IDENTIFIER", "logical_processors=$env:NUMBER_OF_PROCESSORS",
  (rustc -Vv), (cargo -V), 'Host background load and thermal state uncontrolled; alternating order; no warmup excluded.') | Set-Content (Join-Path $output 'host.txt')
foreach ($path in $inputs) { "$path=$(Git @('rev-parse', "${revision}:$path"))" | Add-Content (Join-Path $output 'host.txt') }
foreach ($path in $overlay) { "$path=$((Get-FileHash (Join-Path $root $path) -Algorithm SHA256).Hash)" | Add-Content (Join-Path $output 'host.txt') }
$executables = @{}
foreach ($side in @('before', 'after')) {
    $checkout = Join-Path $output $side
    Git @('worktree', 'add', '--detach', $checkout, $(if ($side -eq 'before') { $before } else { $revision })) | Out-Null
    foreach ($path in $overlay) {
        $destination = Join-Path $checkout $path
        New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
        Copy-Item -LiteralPath (Join-Path $root $path) -Destination $destination
    }
    Copy-Item -LiteralPath (Join-Path $root 'benchmarks/senior-grand-prix/Cargo.lock') -Destination (Join-Path $checkout 'Cargo.lock')
    Push-Location $checkout
    $priorTarget = $env:CARGO_TARGET_DIR
    try {
        $env:CARGO_TARGET_DIR = Join-Path $checkout 'target'
        & cargo test --offline --locked --lib --no-run --message-format=json 2> (Join-Path $output "$side-build.log") | Set-Content (Join-Path $output "$side-build.jsonl")
        if ($LASTEXITCODE -ne 0) { throw "Build failed: $side" }
        $artifacts = Get-Content (Join-Path $output "$side-build.jsonl") | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.profile.test -and $_.executable }
        if (@($artifacts).Count -ne 1) { throw 'Expected one library test executable' }
        $executables[$side] = $artifacts.executable
        "$side binary_sha256=$((Get-FileHash $artifacts.executable -Algorithm SHA256).Hash)" | Add-Content (Join-Path $output 'host.txt')
    } finally { $env:CARGO_TARGET_DIR = $priorTarget; Pop-Location }
}
$previousManifest = $env:ACCEPTANCE_MANIFEST
try {
    for ($run = 1; $run -le $Runs; $run++) {
        $order = if ($run % 2) { @('before', 'after') } else { @('after', 'before') }
        foreach ($side in $order) {
            $env:ACCEPTANCE_MANIFEST = if ($run -eq 1) { Join-Path $output "$side-inputs.txt" } else { $null }
            & $executables[$side] controlled_acceptance --ignored --nocapture --test-threads=1 | Set-Content (Join-Path $output "$side-$run.txt")
            if ($LASTEXITCODE -ne 0) { throw "Benchmark failed: $side / $run" }
        }
    }
} finally { $env:ACCEPTANCE_MANIFEST = $previousManifest }
if ((Get-FileHash (Join-Path $output 'before-inputs.txt')).Hash -ne (Get-FileHash (Join-Path $output 'after-inputs.txt')).Hash) { throw 'Runtime request manifests differ' }
"Retained observations, exact requests and detached instrumented checkouts: $output"
