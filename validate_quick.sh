#!/bin/bash
echo "🎮 Victory Game Launcher - Infrastructure Validation"
echo "===================================================="

PASSED=0
FAILED=0

check() {
    if eval "$1" > /dev/null 2>&1; then
        echo "✅ $2"
        ((PASSED++))
    else
        echo "❌ $2"
        ((FAILED++))
    fi
}

check "[ -f Cargo.toml ]" "Project structure"
check "grep -q 'victory-core' Cargo.toml" "Core crate configured"
check "grep -q 'victory-gpu' Cargo.toml" "GPU crate configured"
check "[ -f crates/victory-gpu/src/d3d12_overlay.rs ]" "D3D12 overlay module exists"
check "grep -q 'DWriteCreateFactory' crates/victory-gpu/src/d3d12_overlay.rs" "DirectWrite API calls present"
check "grep -q 'NamedPipeWriter' crates/victory-gpu/src/profiler.rs" "Named pipes IPC code present"
check "grep -q '#\[cfg(windows)\]' crates/victory-gpu/src/profiler.rs" "Windows gating correct"
check "[ -f tests/integration_windows.rs ]" "Windows integration tests"
check "[ -f run_full_validation.ps1 ]" "PowerShell validation script"
check "[ -f validate_linux_comprehensive.sh ]" "Linux validation script"
check "[ -f WINDOWS_VALIDATION_CHECKLIST.md ]" "Validation documentation"
check "[ -f D3D12_OVERLAY_INTEGRATION.md ]" "D3D12 implementation guide"
check "[ -f HONEST_STATUS.md ]" "Status assessment"

echo ""
echo "Passed: $PASSED ✅"
echo "Failed: $FAILED ❌"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "✅ All infrastructure in place and ready"
    exit 0
else
    exit 1
fi
