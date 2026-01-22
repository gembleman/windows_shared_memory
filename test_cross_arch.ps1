# Cross-Architecture Shared Memory Test Script
# Tests communication between 64-bit and 32-bit processes
#
# Prerequisites:
# - Rust toolchain with both targets installed:
#   rustup target add i686-pc-windows-msvc
#   rustup target add x86_64-pc-windows-msvc
#
# Usage:
#   .\test_cross_arch.ps1
#   .\test_cross_arch.ps1 -SkipBuild   # Skip build step if already built

param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host " Cross-Architecture Shared Memory Test" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Define paths
$projectRoot = $PSScriptRoot
$target64 = "x86_64-pc-windows-msvc"
$target32 = "i686-pc-windows-msvc"

$server64 = "$projectRoot\target\$target64\release\examples\server.exe"
$client64 = "$projectRoot\target\$target64\release\examples\client.exe"
$server32 = "$projectRoot\target\$target32\release\examples\server.exe"
$client32 = "$projectRoot\target\$target32\release\examples\client.exe"

# Build function
function Build-Target {
    param([string]$Target)

    Write-Host "Building for $Target..." -ForegroundColor Yellow
    cargo build --release --examples --target $Target
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed for $Target" -ForegroundColor Red
        Write-Host "Make sure the target is installed: rustup target add $Target" -ForegroundColor Red
        exit 1
    }
    Write-Host "Build successful for $Target" -ForegroundColor Green
    Write-Host ""
}

# Build both targets
if (-not $SkipBuild) {
    Write-Host "=== Building Examples ===" -ForegroundColor Cyan
    Build-Target $target64
    Build-Target $target32
} else {
    Write-Host "Skipping build (using existing binaries)" -ForegroundColor Yellow
    Write-Host ""
}

# Verify binaries exist
Write-Host "=== Verifying Binaries ===" -ForegroundColor Cyan
$binaries = @($server64, $client64, $server32, $client32)
foreach ($bin in $binaries) {
    if (Test-Path $bin) {
        Write-Host "  Found: $bin" -ForegroundColor Green
    } else {
        Write-Host "  Missing: $bin" -ForegroundColor Red
        Write-Host "Please build first: .\test_cross_arch.ps1" -ForegroundColor Red
        exit 1
    }
}
Write-Host ""

# Test function
function Run-Test {
    param(
        [string]$TestName,
        [string]$ServerPath,
        [string]$ClientPath,
        [string]$ShmName
    )

    Write-Host "=== $TestName ===" -ForegroundColor Cyan
    Write-Host "Server: $ServerPath" -ForegroundColor Gray
    Write-Host "Client: $ClientPath" -ForegroundColor Gray
    Write-Host ""

    # Start server in background
    $serverProcess = Start-Process -FilePath $ServerPath -ArgumentList $ShmName -PassThru -NoNewWindow -RedirectStandardOutput "$projectRoot\server_output.txt" -RedirectStandardError "$projectRoot\server_error.txt"

    # Wait a moment for server to initialize
    Start-Sleep -Milliseconds 1000

    # Check if server is still running
    if ($serverProcess.HasExited) {
        Write-Host "ERROR: Server exited prematurely" -ForegroundColor Red
        Get-Content "$projectRoot\server_output.txt"
        Get-Content "$projectRoot\server_error.txt"
        return $false
    }

    # Start client
    $clientProcess = Start-Process -FilePath $ClientPath -ArgumentList $ShmName -PassThru -NoNewWindow -RedirectStandardOutput "$projectRoot\client_output.txt" -RedirectStandardError "$projectRoot\client_error.txt"

    # Wait for client to finish (with timeout)
    $clientExited = $clientProcess.WaitForExit(30000)
    if (-not $clientExited) {
        Write-Host "ERROR: Client timed out" -ForegroundColor Red
        $clientProcess.Kill()
        $serverProcess.Kill()
        return $false
    }

    # Wait for server to finish
    $serverExited = $serverProcess.WaitForExit(5000)
    if (-not $serverExited) {
        Write-Host "WARNING: Server did not exit gracefully, killing..." -ForegroundColor Yellow
        $serverProcess.Kill()
    }

    # Display output
    Write-Host "--- Server Output ---" -ForegroundColor Gray
    Get-Content "$projectRoot\server_output.txt"
    $serverErrors = Get-Content "$projectRoot\server_error.txt"
    if ($serverErrors) {
        Write-Host "--- Server Errors ---" -ForegroundColor Red
        Write-Host $serverErrors
    }

    Write-Host ""
    Write-Host "--- Client Output ---" -ForegroundColor Gray
    Get-Content "$projectRoot\client_output.txt"
    $clientErrors = Get-Content "$projectRoot\client_error.txt"
    if ($clientErrors) {
        Write-Host "--- Client Errors ---" -ForegroundColor Red
        Write-Host $clientErrors
    }

    # Check exit codes
    $success = ($serverProcess.ExitCode -eq 0) -and ($clientProcess.ExitCode -eq 0)

    Write-Host ""
    if ($success) {
        Write-Host "RESULT: PASSED" -ForegroundColor Green
    } else {
        Write-Host "RESULT: FAILED (Server exit: $($serverProcess.ExitCode), Client exit: $($clientProcess.ExitCode))" -ForegroundColor Red
    }
    Write-Host ""

    # Cleanup temp files
    Remove-Item "$projectRoot\server_output.txt" -ErrorAction SilentlyContinue
    Remove-Item "$projectRoot\server_error.txt" -ErrorAction SilentlyContinue
    Remove-Item "$projectRoot\client_output.txt" -ErrorAction SilentlyContinue
    Remove-Item "$projectRoot\client_error.txt" -ErrorAction SilentlyContinue

    return $success
}

# Run tests
$allPassed = $true

# Test 1: 64-bit Server + 32-bit Client
$result1 = Run-Test -TestName "Test 1: 64-bit Server + 32-bit Client" -ServerPath $server64 -ClientPath $client32 -ShmName "Local\CrossArchTest1"
$allPassed = $allPassed -and $result1

# Small delay between tests
Start-Sleep -Seconds 1

# Test 2: 32-bit Server + 64-bit Client
$result2 = Run-Test -TestName "Test 2: 32-bit Server + 64-bit Client" -ServerPath $server32 -ClientPath $client64 -ShmName "Local\CrossArchTest2"
$allPassed = $allPassed -and $result2

# Small delay between tests
Start-Sleep -Seconds 1

# Test 3: 64-bit Server + 64-bit Client (same architecture baseline)
$result3 = Run-Test -TestName "Test 3: 64-bit Server + 64-bit Client (baseline)" -ServerPath $server64 -ClientPath $client64 -ShmName "Local\CrossArchTest3"
$allPassed = $allPassed -and $result3

# Small delay between tests
Start-Sleep -Seconds 1

# Test 4: 32-bit Server + 32-bit Client (same architecture baseline)
$result4 = Run-Test -TestName "Test 4: 32-bit Server + 32-bit Client (baseline)" -ServerPath $server32 -ClientPath $client32 -ShmName "Local\CrossArchTest4"
$allPassed = $allPassed -and $result4

# Final summary
Write-Host "============================================" -ForegroundColor Cyan
Write-Host " Test Summary" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "Test 1 (64-bit Server + 32-bit Client): $(if($result1){'PASSED'}else{'FAILED'})" -ForegroundColor $(if($result1){'Green'}else{'Red'})
Write-Host "Test 2 (32-bit Server + 64-bit Client): $(if($result2){'PASSED'}else{'FAILED'})" -ForegroundColor $(if($result2){'Green'}else{'Red'})
Write-Host "Test 3 (64-bit Server + 64-bit Client): $(if($result3){'PASSED'}else{'FAILED'})" -ForegroundColor $(if($result3){'Green'}else{'Red'})
Write-Host "Test 4 (32-bit Server + 32-bit Client): $(if($result4){'PASSED'}else{'FAILED'})" -ForegroundColor $(if($result4){'Green'}else{'Red'})
Write-Host ""

if ($allPassed) {
    Write-Host "ALL TESTS PASSED!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "SOME TESTS FAILED!" -ForegroundColor Red
    exit 1
}
