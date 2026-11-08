$ErrorActionPreference = 'Stop'
$root = Join-Path $PSScriptRoot "../../target/acceptance-runner-test-$([guid]::NewGuid())"
$directory = Join-Path $root 'benchmarks/optimizer-acceptance'
New-Item -ItemType Directory -Force -Path $directory | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'run.ps1') $directory
foreach ($path in @('tests/support/workflows.rs', 'tests/support/generation_defaults.rs', 'tests/support/senior_benchmarks.rs', 'tests/fixtures/grand-prix-2026-27/input.tsv', 'rust-toolchain.toml', 'Cargo.toml', 'benchmarks/senior-grand-prix/Cargo.lock')) {
    $destination = Join-Path $root $path
    New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
    'fixture' | Set-Content $destination
}
& git.exe -C $root init --quiet
& git.exe -C $root add -f .
& git.exe -C $root -c user.name=BenchmarkTest -c user.email=benchmark@example.invalid commit --quiet -m fixture
if ($LASTEXITCODE -ne 0) { throw 'Fixture commit failed' }
$baseline = & git.exe -C $root rev-parse HEAD
$runner = Join-Path $directory 'run.ps1'
function Check([string]$Expected, [bool]$Pass, [string]$Message) {
    $result = & (Join-Path $PSHOME 'pwsh.exe') -NoProfile -File $runner -ExpectedRevision $Expected -Baseline $baseline -VerifyOnly 2>&1
    if (($LASTEXITCODE -eq 0) -ne $Pass -or ($result -join "`n") -notmatch $Message) { throw "Unexpected result: $result" }
}
Check $baseline $true 'Verified revisions'
Check ('0' * 40) $false 'git failed'
'dirty' | Set-Content (Join-Path $root 'untracked.txt')
Check $baseline $false 'clean tracked/untracked'
Remove-Item -LiteralPath (Join-Path $root 'untracked.txt')
'changed' | Set-Content (Join-Path $root 'tests/support/workflows.rs')
Check $baseline $false 'clean tracked/untracked'
& git.exe -C $root add tests/support/workflows.rs
& git.exe -C $root -c user.name=BenchmarkTest -c user.email=benchmark@example.invalid commit --quiet -m changed
if ($LASTEXITCODE -ne 0) { throw 'Second fixture commit failed' }
$revision = & git.exe -C $root rev-parse HEAD
Check $baseline $false 'differs from HEAD'
Check $revision $false 'Input differs'
'Acceptance runner guards passed; isolated fixture retained under target.'
exit 0
