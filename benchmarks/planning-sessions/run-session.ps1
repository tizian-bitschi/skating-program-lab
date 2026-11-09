param(
    [Parameter(Mandatory)][string]$ExpectedRevision,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(3,20)][int]$Runs = 3
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$revision = & git.exe -C $root rev-parse HEAD
if ($revision -ne $ExpectedRevision) { throw 'Revision differs from HEAD' }
if (& git.exe -C $root status --porcelain --untracked-files=normal) { throw 'Requires clean checkout' }
$exe = Join-Path $root 'target/release/examples/planning_sessions.exe'
if (!(Test-Path -LiteralPath $exe)) { throw 'Build Release example first' }
if ((Test-Path -LiteralPath $OutputDirectory) -and (Get-ChildItem -LiteralPath $OutputDirectory -Force)) { throw 'Output directory must be empty' }
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$output = (Resolve-Path -LiteralPath $OutputDirectory).Path
@("revision=$revision", "runs=$Runs", "binary_sha256=$((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash)", "lock_sha256=$((Get-FileHash -LiteralPath (Join-Path $root 'Cargo.lock') -Algorithm SHA256).Hash)", "machine=$env:COMPUTERNAME", "os=$([Environment]::OSVersion)", "cpu=$env:PROCESSOR_IDENTIFIER", (rustc -Vv), (cargo -V), 'Native Release. Serial fresh processes. No excluded warmup. No concurrent builds/tests/benchmarks. Host background load and thermal state uncontrolled. Windows lifetime peak resident memory sampled after metrics while child waits for ACK.') | Set-Content -LiteralPath (Join-Path $output 'environment.txt')
'new_us,proof_us,first_five_us,active_collection_us,pause_us,resumed_batch_us,alternatives,batches,state,passed,mode,run,peak_working_set_bytes,request_sha256,identity_sha256' | Set-Content -LiteralPath (Join-Path $output 'observations.csv')
$utf8 = [Text.UTF8Encoding]::new($false)
$expectedIdentity = $null
foreach ($run in 1..$Runs) {
    foreach ($mode in @('uninterrupted','paused')) {
        $info = [Diagnostics.ProcessStartInfo]::new()
        $info.FileName = $exe
        $info.ArgumentList.Add($mode)
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardInput = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $process = [Diagnostics.Process]::new()
        $process.StartInfo = $info
        [void]$process.Start()
        $errorRead = $process.StandardError.ReadToEndAsync()
        $lines = [Collections.Generic.List[string]]::new()
        $metric = $null; $request = $null; $identity = $null
        while (!$metric) {
            $read = $process.StandardOutput.ReadLineAsync()
            if (!$read.Wait(120000)) { $process.Kill($true); throw 'WatchdogFailure, never infeasibility' }
            $line = $read.Result
            if ($null -eq $line) { break }
            $lines.Add($line)
            if ($line.StartsWith("REQUEST`t")) { $request = $line.Substring(8) + "`n" }
            if ($line.StartsWith("IDENTITIES`t")) { $identity = $line.Substring(11) + "`n" }
            if ($line.StartsWith("METRIC`t")) { $metric = $line.Substring(7) }
        }
        $process.Refresh(); $peak = $process.PeakWorkingSet64
        if ($metric) { $process.StandardInput.WriteLine('ACK'); $process.StandardInput.Flush() }
        if (!$process.WaitForExit(10000)) { $process.Kill($true); $process.WaitForExit() }
        $label = "$mode-$run"
        [IO.File]::WriteAllText((Join-Path $output "$label.stdout.txt"), ($lines -join "`n") + "`n", $utf8)
        [IO.File]::WriteAllText((Join-Path $output "$label.stderr.txt"), $errorRead.GetAwaiter().GetResult(), $utf8)
        if (!$metric -or $process.ExitCode -ne 0 -or !$identity -or !$request) { throw "Experiment failed: $label" }
        if ($expectedIdentity -and $identity -ne $expectedIdentity) { throw 'Pause/resume or repetition changed complete identity set' }
        $expectedIdentity = $identity
        $requestFile = Join-Path $output "$label.request.txt"
        $identityFile = Join-Path $output "$label.identities.txt"
        [IO.File]::WriteAllText($requestFile, $request, $utf8)
        [IO.File]::WriteAllText($identityFile, $identity, $utf8)
        "$metric,$mode,$run,$peak,$((Get-FileHash -LiteralPath $requestFile -Algorithm SHA256).Hash),$((Get-FileHash -LiteralPath $identityFile -Algorithm SHA256).Hash)" | Add-Content -LiteralPath (Join-Path $output 'observations.csv')
        $process.Dispose()
    }
}
