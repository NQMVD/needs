#!/bin/bash

# Test script to validate the Homebrew formula for needs
# This simulates the key parts of what Homebrew would do

set -e

echo "Testing needs Homebrew formula..."

# Check if we're in the right directory
cd /home/runner/work/needs/needs

# Function to test a formula file
test_formula() {
    local formula_file="$1"
    echo "Testing formula: $formula_file"
    
    # Test: Verify the formula file exists and has correct structure
    echo "  Checking formula structure..."
    if [ -f "$formula_file" ]; then
        echo "  ✓ Formula file exists"
        
        # Check for required fields
        if grep -q "desc" "$formula_file"; then
            echo "  ✓ Description field found"
        else
            echo "  ✗ Description field missing"
            return 1
        fi
        
        if grep -q "homepage" "$formula_file"; then
            echo "  ✓ Homepage field found"
        else
            echo "  ✗ Homepage field missing"
            return 1
        fi
        
        if grep -q "depends_on.*rust" "$formula_file"; then
            echo "  ✓ Rust dependency found"
        else
            echo "  ✗ Rust dependency missing"
            return 1
        fi
        
        if grep -q "def install" "$formula_file"; then
            echo "  ✓ Install method found"
        else
            echo "  ✗ Install method missing"
            return 1
        fi
        
        if grep -q "test do" "$formula_file"; then
            echo "  ✓ Test method found"
        else
            echo "  ✗ Test method missing"
            return 1
        fi
    else
        echo "  ✗ Formula file not found"
        return 1
    fi
    
    # Check Ruby syntax
    echo "  Checking Ruby syntax..."
    if command -v ruby >/dev/null 2>&1; then
        if ruby -c "$formula_file" >/dev/null 2>&1; then
            echo "  ✓ Ruby syntax is valid"
        else
            echo "  ✗ Ruby syntax is invalid"
            return 1
        fi
    else
        echo "  ! Ruby not available, skipping syntax check"
    fi
    
    echo "  ✓ Formula $formula_file is valid"
}

# Test all formula files
echo "Test 1: Testing formula files..."
test_formula "needs.rb"
test_formula "needs-release.rb"
test_formula "needs-advanced.rb"

# Test 2: Verify we can build the project (simulates what Homebrew would do)
echo "Test 2: Building project..."
if cargo build --release; then
    echo "✓ Project builds successfully"
else
    echo "✗ Build failed"
    return 1
fi

# Test 3: Test the binary functionality (simulates formula tests)
echo "Test 3: Testing binary functionality..."
BINARY="./target/release/needs"

if [ -f "$BINARY" ]; then
    echo "✓ Binary exists"
    
    # Test help output
    if $BINARY --help | grep -q "Check if given bin(s) are available in the PATH"; then
        echo "✓ Help text is correct"
    else
        echo "✗ Help text is incorrect"
        return 1
    fi
    
    # Test version output
    if $BINARY --version | grep -q "needs"; then
        echo "✓ Version output is correct"
    else
        echo "✗ Version output is incorrect"
        return 1
    fi
    
    # Test basic functionality with 'sh' (should exist on both macOS and Linux)
    if $BINARY --quiet sh; then
        echo "✓ Basic functionality works"
    else
        echo "✗ Basic functionality failed"
        return 1
    fi
    
    # Test no-versions flag
    if $BINARY --no-versions sh | grep -q "sh"; then
        echo "✓ No-versions flag works"
    else
        echo "✗ No-versions flag failed"
        return 1
    fi
    
    # Test multiple binaries
    if $BINARY --no-versions sh ls | grep -q "sh" && $BINARY --no-versions sh ls | grep -q "ls"; then
        echo "✓ Multiple binaries work"
    else
        echo "✗ Multiple binaries failed"
        return 1
    fi
    
    # Test with non-existent binary
    if $BINARY --no-versions nonexistent_binary_xyz | grep -q "not found"; then
        echo "✓ Non-existent binary handling works"
    else
        echo "✗ Non-existent binary handling failed"
        return 1
    fi
    
else
    echo "✗ Binary not found"
    return 1
fi

# Test 4: Check documentation files
echo "Test 4: Checking documentation..."
if [ -f "HOMEBREW.md" ]; then
    echo "✓ HOMEBREW.md exists"
else
    echo "✗ HOMEBREW.md missing"
fi

if [ -f "INSTALL.md" ]; then
    echo "✓ INSTALL.md exists"
else
    echo "✗ INSTALL.md missing"
fi

echo "All tests passed! The Homebrew formulas should work correctly."