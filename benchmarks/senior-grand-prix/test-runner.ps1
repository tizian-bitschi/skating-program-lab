$ErrorActionPreference = 'Stop'
$testRoot = Join-Path $PSScriptRoot "../../target/runner-test-$([guid]::NewGuid())"
$scriptDirectory = Join-Path $testRoot 'benchmarks/senior-grand-prix'
New-Item -ItemType Directory -Force -Path $scriptDirectory | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'run.ps1') -Destination $scriptDirectory
$fixture = Join-Path $testRoot 'fixture.txt'
'original' | Set-Content $fixture
& git -C $testRoot init --quiet
& git -C $testRoot add .
& git -C $testRoot -c user.name=BenchmarkTest -c user.email=benchmark@example.invalid commit --quiet -m fixture
if ($LASTEXITCODE -ne 0) { throw 'Cannot create isolated runner fixture' }
$revision = & git -C $testRoot rev-parse HEAD
$runner = Join-Path $scriptDirectory 'run.ps1'

function Check-Runner([string]$Expected, [bool]$ShouldPass, [string]$Diagnostic) {
    $output = & (Join-Path $PSHOME 'pwsh.exe') -NoProfile -File $runner -ExpectedRevision $Expected -VerifyOnly 2>&1
    $succeeded = $LASTEXITCODE -eq 0
    if ($succeeded -ne $ShouldPass -or ($output -join "`n") -notmatch $Diagnostic) {
        throw "Unexpected runner outcome: $output"
    }
}

Check-Runner $revision $true 'Verified clean revision'
Check-Runner '0000000000000000000000000000000000000000' $false 'Expected revision is not a known commit'
# An existing but different commit must also fail, before any build or lock mutation.
'second' | Set-Content $fixture
& git -C $testRoot add fixture.txt
& git -C $testRoot -c user.name=BenchmarkTest -c user.email=benchmark@example.invalid commit --quiet -m second
Check-Runner $revision $false 'checked out'
$revision = & git -C $testRoot rev-parse HEAD
'modified' | Set-Content $fixture
Check-Runner $revision $false 'clean tracked/untracked'
'second' | Set-Content $fixture
$untracked = Join-Path $testRoot 'untracked.txt'
'untracked' | Set-Content $untracked
Check-Runner $revision $false 'clean tracked/untracked'
Remove-Item -LiteralPath $untracked
Check-Runner $revision $true 'Verified clean revision'
if (Test-Path -LiteralPath (Join-Path $testRoot 'Cargo.lock')) { throw 'Preflight created a lockfile' }
'Runner preflight checks passed (isolated fixture retained under target).'

# Exercise lock setup/restoration with a real dependency-free build, without timing runs.
@'
[package]
name = "runner-fixture"
version = "0.1.0"
edition = "2021"
'@ | Set-Content (Join-Path $testRoot 'Cargo.toml')
New-Item -ItemType Directory -Path (Join-Path $testRoot 'examples') | Out-Null
'fn main() { println!("fixture-domain"); }' | Set-Content (Join-Path $testRoot 'examples/senior_grand_prix.rs')
"/target/`n/Cargo.lock" | Set-Content (Join-Path $testRoot '.gitignore')
@'
version = 4
[[package]]
name = "runner-fixture"
version = "0.1.0"
'@ | Set-Content (Join-Path $scriptDirectory 'Cargo.lock')
& git -C $testRoot add .
& git -C $testRoot -c user.name=BenchmarkTest -c user.email=benchmark@example.invalid commit --quiet -m setup
if ($LASTEXITCODE -ne 0) { throw 'Cannot commit runner setup fixture' }
$revision = & git -C $testRoot rev-parse HEAD
$rootLock = Join-Path $testRoot 'Cargo.lock'
function Check-Describe([bool]$ShouldPass) {
    $output = & (Join-Path $PSHOME 'pwsh.exe') -NoProfile -File $runner -ExpectedRevision $revision -Describe 2>&1
    if (($LASTEXITCODE -eq 0) -ne $ShouldPass) { throw "Unexpected describe outcome: $output" }
}
Check-Describe $true
if (Test-Path -LiteralPath $rootLock) { throw 'Describe retained an originally absent lock' }
$sentinel = [byte[]](0, 255, 10, 13, 65)
[IO.File]::WriteAllBytes($rootLock, $sentinel)
Check-Describe $true
if ([Convert]::ToBase64String([IO.File]::ReadAllBytes($rootLock)) -ne [Convert]::ToBase64String($sentinel)) {
    throw 'Describe changed an existing lock'
}
# A real compiler failure must also restore the original lock in finally.
'not valid Rust' | Set-Content (Join-Path $testRoot 'examples/senior_grand_prix.rs')
& git -C $testRoot add examples/senior_grand_prix.rs
& git -C $testRoot -c user.name=BenchmarkTest -c user.email=benchmark@example.invalid commit --quiet -m broken
if ($LASTEXITCODE -ne 0) { throw 'Cannot commit failing build fixture' }
$revision = & git -C $testRoot rev-parse HEAD
Check-Describe $false
if ([Convert]::ToBase64String([IO.File]::ReadAllBytes($rootLock)) -ne [Convert]::ToBase64String($sentinel)) {
    throw 'Build failure changed an existing lock'
}
Remove-Item -LiteralPath $rootLock
Check-Describe $false
if (Test-Path -LiteralPath $rootLock) { throw 'Build failure retained an originally absent lock' }
'Runner lock lifecycle checks passed.'
# The last child deliberately fails; do not propagate its exit code to CI.
exit 0
