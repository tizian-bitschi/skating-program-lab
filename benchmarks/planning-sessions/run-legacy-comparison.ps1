param(
    [Parameter(Mandatory)][string]$BaselineBinary,
    [Parameter(Mandatory)][string]$BaselineRevision,
    [Parameter(Mandatory)][string]$CurrentBinary,
    [Parameter(Mandatory)][string]$CurrentRevision,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(3, 20)][int]$Runs = 3,
    [ValidateRange(1, 3600)][int]$WatchdogSeconds = 120
)
$ErrorActionPreference = 'Stop'
$binaries = @{ baseline = (Resolve-Path -LiteralPath $BaselineBinary).Path; current = (Resolve-Path -LiteralPath $CurrentBinary).Path }
if ((Test-Path -LiteralPath $OutputDirectory) -and (Get-ChildItem -LiteralPath $OutputDirectory -Force)) { throw 'Output directory must be empty' }
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$output = (Resolve-Path -LiteralPath $OutputDirectory).Path
@("baseline_revision=$BaselineRevision", "current_revision=$CurrentRevision", "runs=$Runs", "external_watchdog_seconds=$WatchdogSeconds",
    "baseline_binary_sha256=$((Get-FileHash -LiteralPath $binaries.baseline -Algorithm SHA256).Hash)",
    "current_binary_sha256=$((Get-FileHash -LiteralPath $binaries.current -Algorithm SHA256).Hash)",
    "machine=$env:COMPUTERNAME", "os=$([Environment]::OSVersion.VersionString)", "processor=$([Environment]::GetEnvironmentVariable('PROCESSOR_IDENTIFIER'))", "logical_processors=$([Environment]::ProcessorCount)",
    (rustc -Vv), (cargo -V),
    'Native Release, Pumpkin0.5.0; binary source revisions supplied explicitly; build logs and Cargo.lock retained alongside evidence.',
    'Serial fresh process per row; no warmup discarded; compilation and ACK excluded; host background load and thermal state uncontrolled.',
    'proof_us=start to MaximumProved poll, includes initial reconstruction/assessment; alternatives_us=that poll to legacy capped return.',
    'peak_working_set_bytes=Windows process lifetime peak sampled while child waits for ACK after API return.',
    'Request manifests must match byte-for-byte across both revisions and every repetition. Bronze required changes from five nominal alternatives to four distinct exhausted compositions.',
    'WatchdogFailure is experimental termination, never infeasibility. No API deadline or browser latency guarantee.') | Set-Content -LiteralPath (Join-Path $output 'environment.txt')
$header = 'edition,profile,scenario,cancel_phase,outcome,maximum_hundredths,alternatives,alternative_status,domain_us,model_us,first_valid_us,proof_us,alternatives_us,total_us,cancel_response_us,passed,run,peak_working_set_bytes,request_sha256,exit_code'
$header | Set-Content -LiteralPath (Join-Path $output 'observations.csv')
$failures = 0
foreach ($edition in @('baseline', 'current')) {
    foreach ($profileName in @('Bronze', 'Silver', 'Gold', 'Masters', 'Women', 'Men')) {
        for ($run = 1; $run -le $Runs; $run++) {
            foreach ($scenario in @('empty', 'required', 'excluded', 'impossible')) {
                $label = "$edition-$profileName-$scenario-$run"
                $info = [Diagnostics.ProcessStartInfo]::new()
                $info.FileName = $binaries[$edition]
                $info.ArgumentList.Add($profileName)
                $info.ArgumentList.Add($scenario)
                $info.UseShellExecute = $false
                $info.CreateNoWindow = $true
                $info.RedirectStandardInput = $true
                $info.RedirectStandardOutput = $true
                $info.RedirectStandardError = $true
                $process = [Diagnostics.Process]::new()
                $process.StartInfo = $info
                [void]$process.Start()
                $stderr = $process.StandardError.ReadToEndAsync()
                $lines = [Collections.Generic.List[string]]::new()
                $timer = [Diagnostics.Stopwatch]::StartNew()
                $metric = $null
                $request = $null
                $timedOut = $false
                while (!$metric) {
                    $read = $process.StandardOutput.ReadLineAsync()
                    $remaining = [Math]::Max(1, $WatchdogSeconds * 1000 - [int]$timer.ElapsedMilliseconds)
                    if (!$read.Wait($remaining)) { $timedOut = $true; break }
                    $line = $read.Result
                    if ($null -eq $line) { break }
                    $lines.Add($line)
                    if ($line.StartsWith("REQUEST`t")) { $request = $line.Substring(8) + "`n" }
                    if ($line.StartsWith("METRIC`t")) { $metric = $line.Substring(7) }
                }
                $process.Refresh()
                $peak = $process.PeakWorkingSet64
                if ($metric) { $process.StandardInput.WriteLine('ACK'); $process.StandardInput.Flush() }
                else { $process.Kill($true) }
                if (!$process.WaitForExit(10000)) { $process.Kill($true); $process.WaitForExit() }
                [IO.File]::WriteAllLines((Join-Path $output "$label.stdout.txt"), $lines)
                [IO.File]::WriteAllText((Join-Path $output "$label.stderr.txt"), $stderr.GetAwaiter().GetResult())
                $hash = ''
                if ($request) {
                    $manifest = Join-Path $output "$profileName-$scenario.request.txt"
                    if (Test-Path -LiteralPath $manifest) {
                        if ([IO.File]::ReadAllText($manifest) -cne $request) { throw "Request changed: $label" }
                    } else { [IO.File]::WriteAllText($manifest, $request, [Text.UTF8Encoding]::new($false)) }
                    $hash = (Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash
                }
                if (!$metric) {
                    $outcome = if ($timedOut) { 'WatchdogFailure' } else { 'ProcessFailure' }
                    $metric = "$profileName,$scenario,,$outcome,,0,,,,,,,,,false"
                }
                if ($metric.Split(',').Count -ne 15) { throw "Malformed metric: $label" }
                "$edition,$metric,$run,$peak,$hash,$($process.ExitCode)" | Add-Content -LiteralPath (Join-Path $output 'observations.csv')
                if ($metric.Split(',')[14] -ne 'true' -or $process.ExitCode -ne 0) { $failures++; Write-Warning "Retained failure: $label" }
                $process.Dispose()
            }
        }
        Write-Output "Completed $edition $profileName; failures=$failures"
    }
}
if ($failures) { throw "$failures failures; see retained observations" }
Write-Output "Passed $($Runs * 48) serial legacy observations: $output"
