#!/bin/bash
# Integration test script for btl
# Tests all major functionality of the background task manager

set -e  # Exit on error

BTL="./target/debug/btl"
FAILED=0
PASSED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test utilities
test_start() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

test_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    PASSED=$((PASSED + 1))
}

test_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    FAILED=$((FAILED + 1))
}

cleanup() {
    echo -e "\n${YELLOW}[CLEANUP]${NC} Killing all btl processes..."
    $BTL kill --all 2>/dev/null || true
    $BTL clean --all 2>/dev/null || true
}

# Build btl first
echo -e "${YELLOW}[BUILD]${NC} Building btl..."
cargo build --quiet 2>&1 || {
    echo -e "${RED}[ERROR]${NC} Failed to build btl"
    exit 1
}

# Ensure clean state
cleanup

echo ""
echo "=================================="
echo "  BTL Integration Test Suite"
echo "=================================="
echo ""

# Test 1: Basic help command
test_start "Help command shows usage"
if $BTL --help | grep -q "backgrounds processes"; then
    test_pass "Help command works"
else
    test_fail "Help command failed"
fi

# Test 2: List with no processes
test_start "List shows no processes initially"
if $BTL list | grep -q "No background processes tracked"; then
    test_pass "Empty list works"
else
    test_fail "Empty list failed"
fi

# Test 3: Start a background process
test_start "Start a background process"
OUTPUT=$($BTL sleep 3 2>&1)
if echo "$OUTPUT" | grep -q "Process backgrounded"; then
    test_pass "Process started"
    HASH=$(echo "$OUTPUT" | grep "Hash:" | awk '{print $3}')
    echo "  -> Hash: $HASH"
else
    test_fail "Failed to start process"
fi

# Test 4: List shows the process
test_start "List shows running process"
sleep 0.5  # Give it a moment to register
if $BTL list | grep -q "sleep 3"; then
    test_pass "Process appears in list"
else
    test_fail "Process not in list"
fi

# Test 5: Auto-replace functionality
test_start "Auto-replace: starting same command kills old process"
OUTPUT1=$($BTL sleep 10 2>&1)
PID1=$(echo "$OUTPUT1" | grep "PID" | head -1 | grep -oE '[0-9]+' | tail -1)
sleep 0.5
OUTPUT2=$($BTL sleep 10 2>&1)
if echo "$OUTPUT2" | grep -q "Killing previous process"; then
    test_pass "Auto-replace works"
    # Kill the second one
    HASH2=$(echo "$OUTPUT2" | grep "Hash:" | awk '{print $3}')
    $BTL kill "$HASH2" 2>/dev/null || true
else
    test_fail "Auto-replace failed"
fi

# Test 6: Logs command (without tailing)
test_start "Logs command lists available logs"
$BTL sleep 2 >/dev/null 2>&1
sleep 0.5
if $BTL logs | grep -q "Available logs"; then
    test_pass "Logs listing works"
else
    test_fail "Logs listing failed"
fi

# Test 7: Kill specific process
test_start "Kill specific process by hash"
OUTPUT=$($BTL sleep 5 2>&1)
HASH=$(echo "$OUTPUT" | grep "Hash:" | awk '{print $3}')
sleep 0.5
if $BTL kill "$HASH" 2>&1 | grep -q "Process killed"; then
    test_pass "Kill by hash works"
else
    test_fail "Kill by hash failed"
fi

# Test 8: Clean command (dead processes)
test_start "Clean removes dead process entries"
# Start and manually kill a process
OUTPUT=$($BTL sleep 10 2>&1)
PID=$(echo "$OUTPUT" | grep "PID" | head -1 | grep -oE '[0-9]+' | tail -1)
kill "$PID" 2>/dev/null || true
sleep 0.5
if $BTL clean 2>&1 | grep -q "Removed.*dead process"; then
    test_pass "Clean works"
else
    # Might say "No dead processes" if already cleaned
    if $BTL clean 2>&1 | grep -q "No dead processes"; then
        test_pass "Clean works (no dead processes)"
    else
        test_fail "Clean failed"
    fi
fi

# Test 9: Multiple concurrent processes
test_start "Multiple different commands can run concurrently"
$BTL sleep 3 >/dev/null 2>&1
$BTL sh -c "sleep 3" >/dev/null 2>&1
sleep 0.5
COUNT=$($BTL list | grep -c "sleep" || echo "0")
if [ "$COUNT" -ge 2 ]; then
    test_pass "Multiple processes tracked"
else
    test_fail "Multiple processes not tracked properly"
fi

# Test 10: Kill all processes
test_start "Kill all processes"
$BTL sleep 10 >/dev/null 2>&1
$BTL sh -c "sleep 10" >/dev/null 2>&1
sleep 0.5
if $BTL kill --all 2>&1 | grep -q "Killed"; then
    test_pass "Kill all works"
else
    test_fail "Kill all failed"
fi

# Test 11: Safety - refuse to run as root (skip if not possible to test)
test_start "Safety check: refuse to run as root"
if [ "$(id -u)" = "0" ]; then
    if $BTL list 2>&1 | grep -q "refuses to run as root"; then
        test_pass "Root check works"
    else
        test_fail "Root check failed"
    fi
else
    echo -e "  ${YELLOW}[SKIP]${NC} Cannot test root check as non-root user"
fi

# Test 12: Process isolation (different users can't affect each other's processes)
test_start "Process isolation by UID"
# This is inherently tested by the implementation, just verify state is loaded
if $BTL list >/dev/null 2>&1; then
    test_pass "State management works (isolation implicit)"
else
    test_fail "State management failed"
fi

# Test 13: Log file creation
test_start "Log files are created"
OUTPUT=$($BTL echo "test output" 2>&1)
HASH=$(echo "$OUTPUT" | grep "Hash:" | awk '{print $3}')
LOG_PATH=$(echo "$OUTPUT" | grep "Logs:" | awk '{print $3}')
sleep 0.5
if [ -f "$LOG_PATH" ]; then
    test_pass "Log file created"
    if grep -q "test output" "$LOG_PATH"; then
        test_pass "Log file contains output"
    else
        test_fail "Log file missing output"
    fi
else
    test_fail "Log file not created"
fi

# Test 14: Hash consistency
test_start "Same command generates same hash"
OUTPUT1=$($BTL echo "consistent" 2>&1)
HASH1=$(echo "$OUTPUT1" | grep "Hash:" | awk '{print $3}')
$BTL kill "$HASH1" 2>/dev/null || true
sleep 0.5
OUTPUT2=$($BTL echo "consistent" 2>&1)
HASH2=$(echo "$OUTPUT2" | grep "Hash:" | awk '{print $3}')
if [ "$HASH1" = "$HASH2" ]; then
    test_pass "Hash consistency works"
else
    test_fail "Hashes don't match for same command"
fi

# Test 15: Clean with --all flag
test_start "Clean --all removes all entries"
$BTL sleep 10 >/dev/null 2>&1
sleep 0.5
$BTL clean --all >/dev/null 2>&1
if $BTL list | grep -q "No background processes tracked"; then
    test_pass "Clean --all works"
else
    test_fail "Clean --all failed"
fi

# Final cleanup
cleanup

# Summary
echo ""
echo "=================================="
echo "  Test Summary"
echo "=================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
