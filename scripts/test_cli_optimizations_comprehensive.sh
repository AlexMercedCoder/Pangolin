#!/bin/bash
# Comprehensive live test script for CLI optimization commands
# Tests all 5 new commands with edge cases and error handling

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_URL="${PANGOLIN_URL:-http://localhost:8080}"
ADMIN_CLI="./pangolin/target/debug/pangolin-admin"
TEST_CATALOG="${TEST_CATALOG:-test_catalog}"
TEST_WAREHOUSE="${TEST_WAREHOUSE:-test_warehouse}"

# Counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
print_header() {
    echo ""
    echo "======================================================================="
    echo "$1"
    echo "======================================================================="
}

print_test() {
    echo ""
    echo -e "${YELLOW}>>> Test $TESTS_RUN: $1${NC}"
}

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((TESTS_FAILED++))
}

run_test() {
    ((TESTS_RUN++))
    print_test "$1"
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    if [ ! -f "$ADMIN_CLI" ]; then
        echo -e "${RED}Error: CLI binary not found at $ADMIN_CLI${NC}"
        echo "Please build it first: cd pangolin && cargo build --bin pangolin-admin"
        exit 1
    fi
    
    # Check if API is running
    if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
        echo -e "${RED}Error: API server not responding at $API_URL${NC}"
        echo "Please start the API server first: cd pangolin && cargo run --bin pangolin_api"
        exit 1
    fi
    
    echo -e "${GREEN}✓ All prerequisites met${NC}"
}

# Test 1: Dashboard Statistics
test_stats() {
    run_test "Dashboard Statistics - Basic"
    
    OUTPUT=$($ADMIN_CLI stats 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Dashboard Statistics"; then
        pass "Stats command executed successfully"
    else
        fail "Stats command failed: $OUTPUT"
        return
    fi
    
    # Verify output contains expected fields
    if echo "$OUTPUT" | grep -q "Catalogs:" && \
       echo "$OUTPUT" | grep -q "Warehouses:" && \
       echo "$OUTPUT" | grep -q "Tables:"; then
        pass "Stats output contains all expected fields"
    else
        fail "Stats output missing expected fields"
    fi
}

# Test 2: Catalog Summary - Success Case
test_catalog_summary_success() {
    run_test "Catalog Summary - Existing Catalog"
    
    OUTPUT=$($ADMIN_CLI catalog-summary "$TEST_CATALOG" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Catalog: $TEST_CATALOG"; then
        pass "Catalog summary retrieved successfully"
    else
        fail "Catalog summary failed: $OUTPUT"
        return
    fi
    
    # Verify output format
    if echo "$OUTPUT" | grep -q "Namespaces:" && \
       echo "$OUTPUT" | grep -q "Tables:" && \
       echo "$OUTPUT" | grep -q "Branches:"; then
        pass "Catalog summary contains all expected fields"
    else
        fail "Catalog summary missing expected fields"
    fi
}

# Test 3: Catalog Summary - Not Found
test_catalog_summary_not_found() {
    run_test "Catalog Summary - Non-existent Catalog"
    
    OUTPUT=$($ADMIN_CLI catalog-summary "nonexistent_catalog_xyz" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -qi "error\|failed\|not found"; then
        pass "Catalog summary correctly handles non-existent catalog"
    else
        fail "Catalog summary should return error for non-existent catalog"
    fi
}

# Test 4: Search - Basic Query
test_search_basic() {
    run_test "Asset Search - Basic Query"
    
    OUTPUT=$($ADMIN_CLI search "test" --limit 10 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Found.*results"; then
        pass "Search command executed successfully"
    else
        fail "Search command failed: $OUTPUT"
    fi
}

# Test 5: Search - With Catalog Filter
test_search_with_filter() {
    run_test "Asset Search - With Catalog Filter"
    
    OUTPUT=$($ADMIN_CLI search "test" --catalog "$TEST_CATALOG" --limit 5 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Found.*results"; then
        pass "Search with catalog filter executed successfully"
    else
        fail "Search with catalog filter failed: $OUTPUT"
    fi
}

# Test 6: Search - Empty Results
test_search_empty() {
    run_test "Asset Search - No Results"
    
    OUTPUT=$($ADMIN_CLI search "zzz_nonexistent_xyz_123" --limit 10 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Found 0 results"; then
        pass "Search correctly handles no results"
    else
        fail "Search should return 0 results for non-existent query"
    fi
}

# Test 7: Validate Names - Available Catalogs
test_validate_available() {
    run_test "Name Validation - Available Names"
    
    UNIQUE_NAME="test_catalog_$(date +%s)"
    OUTPUT=$($ADMIN_CLI validate catalog "$UNIQUE_NAME" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Available"; then
        pass "Validation correctly identifies available name"
    else
        fail "Validation failed to identify available name: $OUTPUT"
    fi
}

# Test 8: Validate Names - Taken Catalogs
test_validate_taken() {
    run_test "Name Validation - Taken Names"
    
    OUTPUT=$($ADMIN_CLI validate catalog "$TEST_CATALOG" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Taken"; then
        pass "Validation correctly identifies taken name"
    else
        fail "Validation failed to identify taken name: $OUTPUT"
    fi
}

# Test 9: Validate Names - Multiple Names
test_validate_multiple() {
    run_test "Name Validation - Multiple Names"
    
    UNIQUE_NAME="new_catalog_$(date +%s)"
    OUTPUT=$($ADMIN_CLI validate catalog "$TEST_CATALOG" "$UNIQUE_NAME" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Name validation results"; then
        pass "Validation handles multiple names successfully"
    else
        fail "Validation failed with multiple names: $OUTPUT"
    fi
}

# Test 10: Bulk Delete - With Confirm Flag
test_bulk_delete_confirm() {
    run_test "Bulk Delete - With Confirm Flag"
    
    # Use fake UUIDs that won't exist
    FAKE_UUID1="00000000-0000-0000-0000-000000000001"
    FAKE_UUID2="00000000-0000-0000-0000-000000000002"
    
    OUTPUT=$($ADMIN_CLI bulk-delete --ids "$FAKE_UUID1,$FAKE_UUID2" --confirm 2>&1 || true)
    
    if echo "$OUTPUT" | grep -q "Bulk delete completed"; then
        pass "Bulk delete executed with confirm flag"
    else
        fail "Bulk delete failed: $OUTPUT"
    fi
}

# Main execution
main() {
    print_header "CLI Optimization Commands - Comprehensive Live Test"
    echo "API URL: $API_URL"
    echo "CLI Binary: $ADMIN_CLI"
    echo "Test Catalog: $TEST_CATALOG"
    echo "Test Warehouse: $TEST_WAREHOUSE"
    
    check_prerequisites
    
    print_header "Running Tests"
    
    # Dashboard Stats Tests
    test_stats
    
    # Catalog Summary Tests
    test_catalog_summary_success
    test_catalog_summary_not_found
    
    # Search Tests
    test_search_basic
    test_search_with_filter
    test_search_empty
    
    # Validation Tests
    test_validate_available
    test_validate_taken
    test_validate_multiple
    
    # Bulk Delete Tests
    test_bulk_delete_confirm
    
    # Summary
    print_header "Test Summary"
    echo "Total Tests Run: $TESTS_RUN"
    echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo ""
        echo -e "${GREEN}======================================================================="
        echo "✅ ALL TESTS PASSED!"
        echo "=======================================================================${NC}"
        exit 0
    else
        echo ""
        echo -e "${RED}======================================================================="
        echo "❌ SOME TESTS FAILED"
        echo "=======================================================================${NC}"
        exit 1
    fi
}

# Run main
main
