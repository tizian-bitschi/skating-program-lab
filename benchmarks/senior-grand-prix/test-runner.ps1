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
