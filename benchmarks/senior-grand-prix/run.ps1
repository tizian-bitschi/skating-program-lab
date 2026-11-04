$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '../..')
try {
    # Preserve the caller's lockfile; the benchmark uses a reviewed dependency snapshot.
    $lockPath = Join-Path (Get-Location) 'Cargo.lock'
    $previousLock = if (Test-Path -LiteralPath $lockPath) { [IO.File]::ReadAllBytes($lockPath) } else { $null }
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Cargo.lock') -Destination $lockPath
    $metadata = @(
        "git=$(git rev-parse HEAD)", "os=$([Environment]::OSVersion.VersionString)",
        "processor=$env:PROCESSOR_IDENTIFIER", "logical_processors=$env:NUMBER_OF_PROCESSORS",
        (rustc -Vv), (cargo -V), 'pumpkin=0.5.0; default RNG seed=42',
        'Compilation excluded; one observation per configuration; other host activity uncontrolled.'
    )
    $metadata | Set-Content docs/senior-grand-prix-host.txt
    foreach ($profile in @('debug', 'release')) {
        $buildArgs = @('build', '--offline', '--locked', '--example', 'senior_grand_prix')
        if ($profile -eq 'release') { $buildArgs += '--release' }
        & cargo @buildArgs
        if ($LASTEXITCODE -ne 0) { throw "Build failed: $profile" }
        $binary = Join-Path (Get-Location) "target/$profile/examples/senior_grand_prix.exe"
        foreach ($budget in @(250, 1000, 5000, 30000)) {
            & $binary $budget | Set-Content "docs/senior-grand-prix-$profile-$budget.csv"
            if ($LASTEXITCODE -ne 0) { throw "Benchmark failed: $profile / $budget" }
        }
    }
} finally {
    if ($null -ne $previousLock) { [IO.File]::WriteAllBytes($lockPath, $previousLock) }
    elseif (Test-Path -LiteralPath $lockPath) { Remove-Item -LiteralPath $lockPath }
    Pop-Location
}
