#!/bin/bash

# Update Install E2E Test for tx3up
# This test verifies that tx3up can update an existing installation of the toolchain.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
TEST_NAME="update_install"
TEMP_DIR=$(mktemp -d)
export TX3_ROOT_DIR="$TEMP_DIR/tx3_test"
export TX3_CHANNEL="${TX3_CHANNEL:-stable}"

echo -e "${YELLOW}Starting $TEST_NAME test...${NC}"
echo "TX3_ROOT_DIR: $TX3_ROOT_DIR"
echo "TX3_CHANNEL: $TX3_CHANNEL"
echo "Temp directory: $TEMP_DIR"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up test artifacts...${NC}"
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Function to check if a file exists and is executable
check_executable() {
    local file_path="$1"
    local description="$2"
    
    if [[ -f "$file_path" && -x "$file_path" ]]; then
        echo -e "${GREEN}✓ $description found and executable: $file_path${NC}"
        return 0
    else
        echo -e "${RED}✗ $description not found or not executable: $file_path${NC}"
        return 1
    fi
}

# Function to check if a directory exists
check_directory() {
    local dir_path="$1"
    local description="$2"
    
    if [[ -d "$dir_path" ]]; then
        echo -e "${GREEN}✓ $description exists: $dir_path${NC}"
        return 0
    else
        echo -e "${RED}✗ $description does not exist: $dir_path${NC}"
        return 1
    fi
}

# Function to check if a file exists
check_file() {
    local file_path="$1"
    local description="$2"
    
    if [[ -f "$file_path" ]]; then
        echo -e "${GREEN}✓ $description exists: $file_path${NC}"
        return 0
    else
        echo -e "${RED}✗ $description does not exist: $file_path${NC}"
        return 1
    fi
}

# Function to get file modification time
get_file_mtime() {
    local file_path="$1"
    if [[ -f "$file_path" ]]; then
        stat -c %Y "$file_path" 2>/dev/null || stat -f %m "$file_path" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to count files in directory
count_files() {
    local dir_path="$1"
    if [[ -d "$dir_path" ]]; then
        find "$dir_path" -type f 2>/dev/null | wc -l
    else
        echo "0"
    fi
}

# Main test execution
main() {
    echo -e "${YELLOW}Step 1: Performing initial installation${NC}"
    
    # Ensure clean state
    if [[ -e "$TX3_ROOT_DIR" ]]; then
        rm -rf "$TX3_ROOT_DIR"
    fi
    
    # Run initial installation
    if ! ./target/release/tx3up; then
        echo -e "${RED}✗ Initial tx3up installation failed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Initial installation completed${NC}"
    
    # Verify initial installation
    check_directory "$TX3_ROOT_DIR" "TX3 root directory"
    check_directory "$TX3_ROOT_DIR/$TX3_CHANNEL" "Channel directory ($TX3_CHANNEL)"
    check_directory "$TX3_ROOT_DIR/$TX3_CHANNEL/bin" "Binary directory"
    check_file "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json" "Manifest file"
    
    echo -e "${YELLOW}Step 2: Recording initial state${NC}"
    
    # Record initial state
    initial_manifest_mtime=$(get_file_mtime "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json")
    initial_bin_count=$(count_files "$TX3_ROOT_DIR/$TX3_CHANNEL/bin")
    
    echo "Initial manifest mtime: $initial_manifest_mtime"
    echo "Initial binary count: $initial_bin_count"
    
    # Create a test marker file to verify update behavior
    test_marker="$TX3_ROOT_DIR/test_marker.txt"
    echo "test_marker_$(date +%s)" > "$test_marker"
    
    # Wait a moment to ensure different timestamps
    sleep 2
    
    echo -e "${YELLOW}Step 3: Running update installation${NC}"
    
    # Run the installer again (should update existing installation)
    if ! ./target/release/tx3up; then
        echo -e "${RED}✗ tx3up update installation failed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Update installation completed${NC}"
    
    echo -e "${YELLOW}Step 4: Verifying update behavior${NC}"
    
    # Verify installation still exists and is functional
    check_directory "$TX3_ROOT_DIR" "TX3 root directory (after update)"
    check_directory "$TX3_ROOT_DIR/$TX3_CHANNEL" "Channel directory (after update)"
    check_directory "$TX3_ROOT_DIR/$TX3_CHANNEL/bin" "Binary directory (after update)"
    check_file "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json" "Manifest file (after update)"
    
    # Check if our test marker still exists (should be preserved during update)
    if [[ -f "$test_marker" ]]; then
        echo -e "${GREEN}✓ Test marker file preserved during update${NC}"
    else
        echo -e "${YELLOW}⚠ Test marker file not preserved (acceptable behavior)${NC}"
    fi
    
    # Record post-update state
    updated_manifest_mtime=$(get_file_mtime "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json")
    updated_bin_count=$(count_files "$TX3_ROOT_DIR/$TX3_CHANNEL/bin")
    
    echo "Updated manifest mtime: $updated_manifest_mtime"
    echo "Updated binary count: $updated_bin_count"
    
    echo -e "${YELLOW}Step 5: Analyzing update results${NC}"
    
    # Check if update had any effect (manifests should be same or newer)
    if [[ $updated_manifest_mtime -ge $initial_manifest_mtime ]]; then
        echo -e "${GREEN}✓ Manifest file timestamp is same or newer after update${NC}"
    else
        echo -e "${RED}✗ Manifest file timestamp is older after update${NC}"
        exit 1
    fi
    
    # Check if binary count is maintained or increased
    if [[ $updated_bin_count -ge $initial_bin_count ]]; then
        echo -e "${GREEN}✓ Binary count maintained or increased: $initial_bin_count → $updated_bin_count${NC}"
    else
        echo -e "${RED}✗ Binary count decreased after update: $initial_bin_count → $updated_bin_count${NC}"
        exit 1
    fi
    
    # Verify that at least some binaries are still executable
    executable_count=$(find "$TX3_ROOT_DIR/$TX3_CHANNEL/bin" -type f -executable 2>/dev/null | wc -l)
    if [[ $executable_count -gt 0 ]]; then
        echo -e "${GREEN}✓ Found $executable_count executable binaries after update${NC}"
    else
        echo -e "${RED}✗ No executable binaries found after update${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Step 6: Testing idempotency${NC}"
    
    # Run installer one more time to test idempotency
    if ! ./target/release/tx3up; then
        echo -e "${RED}✗ tx3up idempotency test failed${NC}"
        exit 1
    fi
    
    # Verify installation is still intact
    final_manifest_mtime=$(get_file_mtime "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json")
    final_bin_count=$(count_files "$TX3_ROOT_DIR/$TX3_CHANNEL/bin")
    
    if [[ $final_bin_count -eq $updated_bin_count ]]; then
        echo -e "${GREEN}✓ Idempotency test passed - binary count unchanged${NC}"
    else
        echo -e "${YELLOW}⚠ Binary count changed during idempotency test: $updated_bin_count → $final_bin_count${NC}"
    fi
    
    echo -e "${GREEN}🎉 Update install test completed successfully!${NC}"
    echo -e "${GREEN}Update test results:${NC}"
    echo -e "${GREEN}  - Initial binary count: $initial_bin_count${NC}"
    echo -e "${GREEN}  - Updated binary count: $updated_bin_count${NC}"
    echo -e "${GREEN}  - Final binary count: $final_bin_count${NC}"
    echo -e "${GREEN}  - Installation directory: $TX3_ROOT_DIR${NC}"
    echo -e "${GREEN}  - Manifest updates: $initial_manifest_mtime → $updated_manifest_mtime → $final_manifest_mtime${NC}"
}

# Run the main test
main "$@"