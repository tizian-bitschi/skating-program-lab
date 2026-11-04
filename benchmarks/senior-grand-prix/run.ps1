param(
    [Parameter(Mandatory = $true)][string]$ExpectedRevision,
    [switch]$Describe,
    [switch]$VerifyOnly,
    [string]$OutputDirectory = 'target/senior-grand-prix'
)

$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '../..')
try {
    $expected = & git rev-parse --verify "$ExpectedRevision^{commit}"
    if ($LASTEXITCODE -ne 0) { throw 'Expected revision is not a known commit' }
    $actual = & git rev-parse HEAD
    if ($LASTEXITCODE -ne 0 -or $actual -ne $expected) { throw "Expected $expected; checked out $actual" }
    $state = & git status --porcelain --untracked-files=normal
    if ($LASTEXITCODE -ne 0) { throw 'Cannot verify repository state' }
    if ($state) { throw "Benchmark requires a clean tracked/untracked working tree:`n$($state -join "`n")" }
    if ($VerifyOnly) { Write-Output "Verified clean revision $actual"; return }

    # Outputs are ignored build artifacts; measurement runs never dirty the source snapshot.
    New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
    $lockPath = Join-Path (Get-Location) 'Cargo.lock'
    $previousLock = if (Test-Path -LiteralPath $lockPath) { [IO.File]::ReadAllBytes($lockPath) } else { $null }
    try {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Cargo.lock') -Destination $lockPath
        @(
            "git=$actual", "expected_git=$expected", 'dirty=false',
            "lock_sha256=$((Get-FileHash -Algorithm SHA256 -LiteralPath $lockPath).Hash)",
            "os=$([Environment]::OSVersion.VersionString)",
            "processor=$env:PROCESSOR_IDENTIFIER", "logical_processors=$env:NUMBER_OF_PROCESSORS",
            (rustc -Vv), (cargo -V), 'pumpkin=0.5.0; default RNG seed=42',
            'Compilation excluded; one observation per configuration; other host activity uncontrolled.'
        ) | Set-Content (Join-Path $OutputDirectory 'host.txt')
        $profiles = if ($Describe) { @('debug') } else { @('debug', 'release') }
        foreach ($profile in $profiles) {
            $buildArgs = @('build', '--offline', '--locked', '--example', 'senior_grand_prix')
            if ($profile -eq 'release') { $buildArgs += '--release' }
            & cargo @buildArgs
            if ($LASTEXITCODE -ne 0) { throw "Build failed: $profile" }
            $binary = Join-Path (Get-Location) "target/$profile/examples/senior_grand_prix.exe"
            if ($Describe) {
                & $binary --describe | Set-Content (Join-Path $OutputDirectory 'domains.txt')
                if ($LASTEXITCODE -ne 0) { throw 'Domain description failed' }
            } else {
                foreach ($budget in @(250, 1000, 5000, 30000)) {
                    & $binary $budget | Set-Content (Join-Path $OutputDirectory "$profile-$budget.csv")
                    if ($LASTEXITCODE -ne 0) { throw "Benchmark failed: $profile / $budget" }
                }
            }
        }
    } finally {
        if ($null -ne $previousLock) { [IO.File]::WriteAllBytes($lockPath, $previousLock) }
        elseif (Test-Path -LiteralPath $lockPath) { Remove-Item -LiteralPath $lockPath }
    }
} finally {
    Pop-Location
}
