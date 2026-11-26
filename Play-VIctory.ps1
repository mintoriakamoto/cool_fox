# Launch VIctory with auto-tuned staging for NVIDIA compute-class cards (CMP 170HX / RTX) on Windows.
# Reads GPU, VRAM, PCIe link, RAM from the system; auto-tunes texture and DirectStorage staging.
# Play-VIctory.ps1 --print-env     shows what it chose and why, runs nothing.
# Every VICTORY_* value you export yourself wins over the chosen one.

param([switch]$PrintEnv)

$ErrorActionPreference = "Stop"

# --- this machine -----------------------------------------------------------
$gpu_name = "unknown"
$vram_mb = 0
$link_gen = 0
$link_width = 0

# Try to read GPU model, VRAM, PCIe gen and width from Win32_VideoController and Win32_PnPDevice.
try {
    $gpu = Get-WmiObject Win32_VideoController -ErrorAction SilentlyContinue |
           Where-Object { $_.Name -match "NVIDIA|RTX|CMP" } | Select-Object -First 1
    if ($gpu) {
        $gpu_name = $gpu.Name -replace '\s+', ' '
        $vram_mb = [int]($gpu.AdapterRAM / 1MB)
    }
} catch {}

# PCIe gen and width from PnP (best effort; often unavailable on Windows).
try {
    $pnp = Get-WmiObject Win32_PnPDevice -ErrorAction SilentlyContinue |
           Where-Object { $_.Description -match "NVIDIA|RTX|CMP|PCI Express" } |
           Select-Object -First 1
    if ($pnp) {
        # This is best-effort; often not populated. Defaults below.
    }
} catch {}

# If GPU not detected, try nvidia-smi (if available via WSL or installed separately).
if ($gpu_name -eq "unknown" -or $vram_mb -eq 0) {
    try {
        $nvidia_line = & nvidia-smi --query-gpu=name,memory.total,pcie.link.gen.current,pcie.link.width.current `
                           --format=csv,noheader,nounits 2>$null | Select-Object -First 1
        if ($nvidia_line) {
            $parts = $nvidia_line -split ','
            $gpu_name = ($parts[0] -replace '^\s+|\s+$').Trim()
            $vram_mb = [int]($parts[1] -replace '^\s+|\s+$').Trim()
            $link_gen = [int]($parts[2] -replace '^\s+|\s+$').Trim()
            $link_width = [int]($parts[3] -replace '^\s+|\s+$').Trim()
        }
    } catch {}
}

# System RAM (convert from bytes to MiB).
$ram_mb = [int]((Get-WmiObject Win32_ComputerSystem).TotalPhysicalMemory / 1MB)

# Default link spec if not detected: gen3 x16 (conservative).
if ($link_gen -eq 0) { $link_gen = 3 }
if ($link_width -eq 0) { $link_width = 16 }

# PCIe megabytes per second per lane, by generation; half the link for textures at 30 fps.
$lane_mbs = @{ 1 = 250; 2 = 500; 3 = 985; 4 = 1969; 5 = 3938 }
$lane = $lane_mbs[$link_gen] ?? 985
$link_mbs = $lane * $link_width
$stage_mb = [int]($link_mbs / 2 / 30)
if ($stage_mb -lt 16) { $stage_mb = 16 }
if ($stage_mb -gt 128) { $stage_mb = 128 }

# 32 GB of RAM or less: title uses 40-46 GB, needs smaller staging blocks.
$dstorage_mb = if ($ram_mb -le 34000) { 128 } else { 256 }

# Set environment variable only if not already set by the user.
function Set-Default {
    param([string]$Name, [string]$Value)
    if (-not (Test-Path env:$Name)) {
        [Environment]::SetEnvironmentVariable($Name, $Value, "Process")
    }
}

Set-Default "VICTORY_TEXTURE_STAGE_MB" $stage_mb
Set-Default "VICTORY_DSTORAGE_STAGING_MB" $dstorage_mb
Set-Default "VICTORY_UPLOAD_AHEAD" "1"
Set-Default "VICTORY_TEXTURE_NO_WAIT" "1"
# Belobug's renderer ports (untested on NVIDIA; =0 turns one off). PORT_CBV stays off.
Set-Default "VICTORY_PORT_CONSTANTS" "1"
Set-Default "VICTORY_PORT_DESCRIPTORS" "1"
Set-Default "VICTORY_PORT_GUARD" "1"
Set-Default "VICTORY_PORT_JOURNAL" "1"
Set-Default "VICTORY_FSR_4K" "0"
Set-Default "VICTORY_CACHE_DIR" "$env:LOCALAPPDATA\VIctory\cache-pareto"

function Report {
    Write-Host "gpu: $gpu_name, $vram_mb MB VRAM, PCIe gen$link_gen x$link_width (~$link_mbs MB/s)"
    Write-Host "ram: $ram_mb MB"
    Write-Host "texture staging: $stage_mb MiB a frame; DirectStorage staging: $dstorage_mb MiB"
    if ($ram_mb -le 34000 -and (Get-WmiObject Win32_PageFileUsage).AllocatedBaseSize -lt 16000) {
        Write-Host "WARNING: add 16+ GB of paging file on fast storage (not RAM disk)." -ForegroundColor Yellow
    }
    if ($vram_mb -gt 0 -and $vram_mb -lt 16000) {
        Write-Host "NOTE: $vram_mb MB VRAM is below the 40 GB profile assumption." -ForegroundColor Yellow
    }
    if ($gpu_name -eq "unknown") {
        Write-Host "WARNING: no NVIDIA GPU detected; performance may suffer." -ForegroundColor Yellow
    }
}

if ($PrintEnv) {
    Report
    Get-ChildItem env:VICTORY_* | Sort-Object Name | ForEach-Object { Write-Host "$($_.Name)=$($_.Value)" }
    exit 0
}

Report | Write-Host -ForegroundColor Gray

# --- find and run victory.exe ------------------------------------------------
if (-not (Test-Path env:VICTORY_GAME_DIR)) {
    Write-Error "Set VICTORY_GAME_DIR to your own dump folder containing game_scarlett_master.exe" -ErrorAction Stop
}

$runner = Join-Path $PSScriptRoot "victory-3d-rtx-pareto-residency.exe"
if (-not (Test-Path $runner)) {
    $runner = Join-Path $PSScriptRoot "target\release\victory.exe"
}
if (-not (Test-Path $runner)) {
    Write-Error "Extract the Windows release or run cargo build --release --locked -p victory" -ErrorAction Stop
}

$game_exe = Join-Path $env:VICTORY_GAME_DIR "game_scarlett_master.exe"
if (-not (Test-Path $game_exe)) {
    Write-Error "VICTORY_GAME_DIR does not contain game_scarlett_master.exe" -ErrorAction Stop
}

# Run with all the tuned VICTORY_* variables in the environment.
& $runner run $game_exe --console series-s --execute @args
exit $LASTEXITCODE
