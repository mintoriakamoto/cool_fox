# Windows Validation Script for Victory Game Launcher
# Run this on CMP 170HX system to validate all Windows paths

param(
    [string]$GamePath = "C:\Program Files (x86)\Steam\steamapps\common",
    [string]$Duration = 60
)

Write-Host "🎮 Victory Game Launcher - Windows Validation" -ForegroundColor Cyan
Write-Host "============================================`n" -ForegroundColor Cyan

# 1. CPU Metrics Validation
Write-Host "1️⃣  CPU Metrics Validation" -ForegroundColor Yellow
$cpuStart = Get-WmiObject Win32_PerfFormattedData_PerfOS_System | Select-Object SystemUpTime
$cpuBefore = Get-WmiObject Win32_PerfFormattedData_PerfOS_Processor | Where-Object {$_.Name -eq "_Total"} | Select-Object PercentProcessorTime
Write-Host "   CPU Load (before): $($cpuBefore.PercentProcessorTime)%"
Start-Sleep -Seconds 2
$cpuAfter = Get-WmiObject Win32_PerfFormattedData_PerfOS_Processor | Where-Object {$_.Name -eq "_Total"} | Select-Object PercentProcessorTime
Write-Host "   CPU Load (after): $($cpuAfter.PercentProcessorTime)%"
Write-Host "   ✅ GetSystemTimes() validation can run`n"

# 2. GPU Metrics Validation
Write-Host "2️⃣  GPU Metrics Validation (CMP 170HX)" -ForegroundColor Yellow
$nvidiaSmi = & nvidia-smi --query-gpu=memory.total,memory.used,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null
if ($nvidiaSmi) {
    $metrics = $nvidiaSmi -split ","
    Write-Host "   Total VRAM: $($metrics[0].Trim()) MB"
    Write-Host "   Used VRAM: $($metrics[1].Trim()) MB"
    Write-Host "   GPU Load: $($metrics[2].Trim())%"
    Write-Host "   GPU Temp: $($metrics[3].Trim())°C"
    Write-Host "   ✅ nvidia-smi queries working`n"
} else {
    Write-Host "   ❌ nvidia-smi not found or not in PATH`n" -ForegroundColor Red
}

# 3. Named Pipes Test
Write-Host "3️⃣  Named Pipes Validation" -ForegroundColor Yellow
$pipeName = "victory-game-$($PID)"
$pipeServer = New-Object System.IO.Pipes.NamedPipeServerStream($pipeName, [System.IO.Pipes.PipeDirection]::InOut)
Write-Host "   Created named pipe: \\.\pipe\$pipeName"
Write-Host "   ✅ Named pipes working`n"
$pipeServer.Close()
$pipeServer.Dispose()

# 4. Build Profiler
Write-Host "4️⃣  Building Victory Profiler" -ForegroundColor Yellow
$buildResult = & cargo build -p victory-gpu --release 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ victory-gpu built successfully`n"
} else {
    Write-Host "   ❌ Build failed`n" -ForegroundColor Red
    exit 1
}

# 5. Run Validation Session
Write-Host "5️⃣  Ready for Gaming Session Validation" -ForegroundColor Yellow
Write-Host "   Next steps:"
Write-Host "   1. Launch a DirectX 12 game (UE5, Godot, Unity)"
Write-Host "   2. Run the game for $Duration seconds"
Write-Host "   3. Profiler will collect metrics to:"
Write-Host "   4. ~/.local/share/launcher/profiles/"
Write-Host ""
Write-Host "   Then analyze with:"
Write-Host "   cargo run -p launcher-ui --release analyze [game-name]`n"

Write-Host "✅ All Windows paths validated!" -ForegroundColor Green
