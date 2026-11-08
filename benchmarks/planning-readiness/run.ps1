param(
    [Parameter(Mandatory)][string]$ExpectedRevision,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(3, 20)][int]$Runs = 3,
    [ValidateRange(1, 3600)][int]$WatchdogSeconds = 120
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$revision = & git.exe -C $root rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $revision -ne $ExpectedRevision) { throw 'Expected revision differs from HEAD' }
if (& git.exe -C $root status --porcelain --untracked-files=normal) { throw 'Requires a clean source checkout' }
$exe = Join-Path $root 'target/release/examples/planning_readiness.exe'
if (!(Test-Path -LiteralPath $exe)) { throw 'Build Release example first: cargo build --offline --release --example planning_readiness' }
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$output = (Resolve-Path -LiteralPath $OutputDirectory).Path
$cpu = Get-CimInstance Win32_Processor
$os = Get-CimInstance Win32_OperatingSystem
@("revision=$revision", "runs=$Runs", "external_watchdog_seconds=$WatchdogSeconds", 'No API time/node budget; no repertoire reduction; serial fresh process per run; no excluded warmup.',
  "os=$($os.Caption) $($os.Version) $($os.OSArchitecture)", "cpu=$($cpu.Name)", "cores=$($cpu.NumberOfCores); logical=$($cpu.NumberOfLogicalProcessors)",
  "ram_bytes=$((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory)", (rustc -Vv), (cargo -V),
  "binary_sha256=$((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash)",
  "cargo_lock_sha256=$((Get-FileHash -LiteralPath (Join-Path $root 'Cargo.lock') -Algorithm SHA256).Hash)",
  'Pumpkin=0.5.0; native Release; compilation excluded; host thermal/background load uncontrolled.',
  'Memory=Windows Process.PeakWorkingSet64 sampled after API return while child waits for stdin ACK; includes process/runtime/domain/model. ACK is outside total_us.',
  'Phases=first public control poll. domain_us=start->Model; model_us=Model->Solve; first_valid_us=start->validated IncumbentFound; proof_us=start->MaximumProved (includes validation); alternatives_us=proof->return.',
  'Missing phase=empty CSV field. Cancellation response=first true cancellation poll->API return. PostproofReconstruction cancels reconstruction of a later alternative.',
  'Manifest=one-line Rust Debug serialization of every PlanningSearchRequest field, UTF8 without BOM, final LF; compared byte-for-byte on every repetition.') | Set-Content -LiteralPath (Join-Path $output 'environment.txt')
$header = 'profile,scenario,cancel_phase,outcome,maximum_hundredths,alternatives,alternative_status,domain_us,model_us,first_valid_us,proof_us,alternatives_us,total_us,cancel_response_us,passed,run,peak_working_set_bytes,request_sha256,exit_code'
$header | Set-Content -LiteralPath (Join-Path $output 'observations.csv')
$failures = 0
function Measure-Request([string]$Profile, [string]$Scenario, [string]$Cancel, [int]$Run) {
    $label = "$Profile-$Scenario-$(if ($Cancel) { $Cancel } else { 'continue' })-$Run"
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $exe
    foreach ($arg in @($Profile, $Scenario)) { $info.ArgumentList.Add($arg) }
    if ($Cancel) { $info.ArgumentList.Add($Cancel) }
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
    if ($metric) {
        $process.StandardInput.WriteLine('ACK')
        $process.StandardInput.Flush()
    } else { $process.Kill($true) }
    if (!$process.WaitForExit(10000)) { $process.Kill($true); $process.WaitForExit() }
    $errorText = $stderr.GetAwaiter().GetResult()
    [IO.File]::WriteAllLines((Join-Path $output "$label.stdout.txt"), $lines)
    [IO.File]::WriteAllText((Join-Path $output "$label.stderr.txt"), $errorText)
    $hash = ''
    if ($request) {
        $manifest = Join-Path $output "$Profile-$Scenario.request.txt"
        if (Test-Path -LiteralPath $manifest) {
            if ([IO.File]::ReadAllText($manifest) -cne $request) { throw "Request changed: $label" }
        } else { [IO.File]::WriteAllText($manifest, $request, [Text.UTF8Encoding]::new($false)) }
        $hash = (Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash
    }
    if (!$metric) {
        $outcome = if ($timedOut) { 'WatchdogFailure' } else { 'ProcessFailure' }
        $metric = "$Profile,$Scenario,$Cancel,$outcome,,0,,,,,,,,,false"
    }
    "$metric,$Run,$peak,$hash,$($process.ExitCode)" | Add-Content -LiteralPath (Join-Path $output 'observations.csv')
    $passed = ($metric.Split(',')[14] -eq 'true') -and $process.ExitCode -eq 0
    $process.Dispose()
    if (!$passed) { $script:failures++; Write-Warning "Readiness failure retained: $label" }
}
foreach ($profile in @('Bronze', 'Silver', 'Gold', 'Masters', 'Women', 'Men')) {
    for ($run = 1; $run -le $Runs; $run++) {
        foreach ($scenario in @('empty', 'required', 'excluded', 'impossible')) {
            Measure-Request $profile $scenario '' $run
        }
        foreach ($phase in @('Domain', 'Model', 'Solve', 'IncumbentFound', 'Reconstruction', 'MaximumProved', 'Alternatives', 'PostproofReconstruction')) {
            Measure-Request $profile 'empty' $phase $run
        }
    }
    Write-Output "Completed $profile; cumulative failures=$failures"
}
if ($failures) { throw "$failures readiness failures; see retained observations in $output" }
Write-Output "All requests passed; observations: $output"
