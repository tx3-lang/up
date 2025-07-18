#!/bin/bash

# Fresh Install E2E Test for tx3up
# This test verifies that tx3up can perform a clean installation of the toolchain.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
TEST_NAME="fresh_install"
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

# Main test execution
main() {
    echo -e "${YELLOW}Step 1: Verifying clean state${NC}"
    
    # Ensure the root directory doesn't exist initially
    if [[ -e "$TX3_ROOT_DIR" ]]; then
        echo -e "${RED}✗ TX3_ROOT_DIR already exists, not a fresh install${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Clean state verified - TX3_ROOT_DIR does not exist${NC}"
    
    echo -e "${YELLOW}Step 2: Running tx3up install${NC}"
    
    # Run the installer
    if ! ./target/release/tx3up; then
        echo -e "${RED}✗ tx3up installation failed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ tx3up installation completed${NC}"
    
    echo -e "${YELLOW}Step 3: Verifying installation artifacts${NC}"
    
    # Check if the root directory was created
    check_directory "$TX3_ROOT_DIR" "TX3 root directory"
    
    # Check if the channel directory exists
    check_directory "$TX3_ROOT_DIR/$TX3_CHANNEL" "Channel directory ($TX3_CHANNEL)"
    
    # Check if the bin directory exists
    check_directory "$TX3_ROOT_DIR/$TX3_CHANNEL/bin" "Binary directory"
    
    # Check if manifest file exists
    check_file "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json" "Manifest file"
    
    echo -e "${YELLOW}Step 4: Verifying installation contents${NC}"
    
    # List the contents of the installation for debugging
    echo "Installation directory contents:"
    find "$TX3_ROOT_DIR" -type f -exec ls -la {} \; 2>/dev/null || true
    
    # Check if at least one binary was installed
    bin_count=$(find "$TX3_ROOT_DIR/$TX3_CHANNEL/bin" -type f -executable 2>/dev/null | wc -l)
    if [[ $bin_count -gt 0 ]]; then
        echo -e "${GREEN}✓ Found $bin_count executable binaries in bin directory${NC}"
    else
        echo -e "${RED}✗ No executable binaries found in bin directory${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Step 5: Testing manifest file${NC}"
    
    # Verify the manifest file is valid JSON
    if command -v jq >/dev/null 2>&1; then
        if jq empty "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json" 2>/dev/null; then
            echo -e "${GREEN}✓ Manifest file is valid JSON${NC}"
        else
            echo -e "${RED}✗ Manifest file is not valid JSON${NC}"
            exit 1
        fi
    else
        # Basic JSON validation without jq
        if grep -q '^\s*{' "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json" && grep -q '}\s*$' "$TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json"; then
            echo -e "${GREEN}✓ Manifest file appears to be JSON (basic check)${NC}"
        else
            echo -e "${RED}✗ Manifest file does not appear to be valid JSON${NC}"
            exit 1
        fi
    fi
    
    echo -e "${GREEN}🎉 Fresh install test completed successfully!${NC}"
    echo -e "${GREEN}All installation artifacts verified:${NC}"
    echo -e "${GREEN}  - Root directory: $TX3_ROOT_DIR${NC}"
    echo -e "${GREEN}  - Channel: $TX3_CHANNEL${NC}"
    echo -e "${GREEN}  - Channel directory: $TX3_ROOT_DIR/$TX3_CHANNEL${NC}"
    echo -e "${GREEN}  - Binary directory: $TX3_ROOT_DIR/$TX3_CHANNEL/bin${NC}"
    echo -e "${GREEN}  - Manifest file: $TX3_ROOT_DIR/$TX3_CHANNEL/manifest.json${NC}"
    echo -e "${GREEN}  - Executable binaries: $bin_count${NC}"
}

# Run the main test
main "$@"