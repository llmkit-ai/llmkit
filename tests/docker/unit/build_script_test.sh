#!/bin/bash

# Docker Build Script Unit Tests
# Detailed testing of build script functionality and edge cases

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

main() {
    log_info "Starting Docker Build Script Unit Tests"
    
    # Ensure we're in project root
    cd "$PROJECT_ROOT"
    
    # Run unit tests
    test_build_script_existence
    test_build_script_permissions
    test_help_functionality
    test_argument_parsing
    test_invalid_arguments
    test_environment_loading
    test_docker_buildx_detection
    test_compose_file_selection
    test_build_output_validation
    test_cleanup_behavior
    
    print_test_summary
}

test_build_script_existence() {
    log_test_start "Testing build script existence and structure"
    
    # Test script file exists
    assert_file_exists "$BUILD_SCRIPT" "Build script exists"
    
    # Test script is executable
    if [[ -x "$BUILD_SCRIPT" ]]; then
        log_test_pass "Build script is executable"
    else
        log_test_fail "Build script is not executable"
        return 1
    fi
    
    # Test script has shebang
    local first_line=$(head -n1 "$BUILD_SCRIPT")
    assert_contains "$first_line" "#!/bin/bash" "Build script has bash shebang"
    
    return 0
}

test_build_script_permissions() {
    log_test_start "Testing build script permissions"
    
    local permissions=$(stat -c "%a" "$BUILD_SCRIPT")
    
    # Should be at least 755 (rwxr-xr-x) or 744 (rwxr--r--)
    if [[ "$permissions" -ge 744 ]]; then
        log_test_pass "Build script has appropriate permissions: $permissions"
        return 0
    else
        log_test_fail "Build script permissions too restrictive: $permissions"
        return 1
    fi
}

test_help_functionality() {
    log_test_start "Testing help functionality"
    
    # Test --help flag
    local help_output=$($BUILD_SCRIPT --help 2>&1)
    
    assert_contains "$help_output" "Usage:" "Help shows usage"
    assert_contains "$help_output" "--dev" "Help shows dev option"
    assert_contains "$help_output" "--prod" "Help shows prod option"
    assert_contains "$help_output" "--rebuild" "Help shows rebuild option"
    assert_contains "$help_output" "--no-cache" "Help shows no-cache option"
    
    # Test -h flag
    local help_short_output=$($BUILD_SCRIPT -h 2>&1)
    assert_contains "$help_short_output" "Usage:" "Short help flag works"
    
    return 0
}

test_argument_parsing() {
    log_test_start "Testing build script argument parsing"
    
    # Create a test wrapper that captures parsed arguments without building
    local test_script="/tmp/build_test_wrapper.sh"
    cat > "$test_script" << 'EOF'
#!/bin/bash
# Test wrapper to validate argument parsing without actual building

# Source the original build script but override the building part
BUILD_SCRIPT_CONTENT=$(cat "$1" | sed 's/docker compose.*build.*/echo "BUILD_COMMAND: $BUILD_OPTS"/' | sed 's/docker compose.*up.*/echo "UP_COMMAND: executed"/')

# Create temporary modified script
TEMP_SCRIPT="/tmp/temp_build_script.sh"
echo "$BUILD_SCRIPT_CONTENT" > "$TEMP_SCRIPT"
chmod +x "$TEMP_SCRIPT"

# Run with provided arguments
shift
"$TEMP_SCRIPT" "$@"
EOF
    chmod +x "$test_script"
    
    # Test different argument combinations
    local output
    
    # Test --dev
    output=$("$test_script" "$BUILD_SCRIPT" --dev 2>&1 || true)
    if echo "$output" | grep -q "development"; then
        log_test_pass "Dev argument parsing"
    else
        log_test_fail "Dev argument parsing failed"
    fi
    
    # Test --prod
    output=$("$test_script" "$BUILD_SCRIPT" --prod 2>&1 || true)
    if echo "$output" | grep -q "production"; then
        log_test_pass "Prod argument parsing"
    else
        log_test_fail "Prod argument parsing failed"
    fi
    
    # Cleanup
    rm -f "$test_script" "/tmp/temp_build_script.sh"
    
    return 0
}

test_invalid_arguments() {
    log_test_start "Testing invalid argument handling"
    
    local invalid_args=(
        "--invalid"
        "--unknown-flag"
        "random-arg"
        "--dev --prod"  # conflicting args
    )
    
    for arg in "${invalid_args[@]}"; do
        log_info "Testing invalid argument: $arg"
        
        # Should exit with non-zero status
        if $BUILD_SCRIPT $arg >/dev/null 2>&1; then
            log_test_fail "Build script should reject invalid argument: $arg"
            return 1
        else
            log_info "✓ Correctly rejected: $arg"
        fi
    done
    
    log_test_pass "Invalid argument handling"
    return 0
}

test_environment_loading() {
    log_test_start "Testing environment variable loading"
    
    # Create a temporary .env file for testing
    local test_env_file="/tmp/test.env"
    cat > "$test_env_file" << EOF
TEST_VAR=test_value
BACKEND_PORT=9999
UI_PORT=4000
EOF
    
    # Backup original .env if it exists
    local original_env_backup=""
    if [[ -f "$PROJECT_ROOT/.env" ]]; then
        original_env_backup="/tmp/original.env.backup"
        cp "$PROJECT_ROOT/.env" "$original_env_backup"
    fi
    
    # Use test .env file
    cp "$test_env_file" "$PROJECT_ROOT/.env"
    
    # Test that script loads environment variables
    # We'll check the help output which should show the script runs
    if $BUILD_SCRIPT --help >/dev/null 2>&1; then
        log_test_pass "Environment loading (script executes with custom .env)"
    else
        log_test_fail "Environment loading failed"
    fi
    
    # Restore original .env
    if [[ -n "$original_env_backup" && -f "$original_env_backup" ]]; then
        mv "$original_env_backup" "$PROJECT_ROOT/.env"
    else
        rm -f "$PROJECT_ROOT/.env"
    fi
    
    # Cleanup
    rm -f "$test_env_file"
    
    return 0
}

test_docker_buildx_detection() {
    log_test_start "Testing Docker buildx detection"
    
    # Test script behavior when buildx is available vs not available
    # This is challenging to test without affecting the system
    # So we'll test that the script contains the buildx check logic
    
    if grep -q "docker buildx version" "$BUILD_SCRIPT"; then
        log_test_pass "Build script checks for Docker buildx"
    else
        log_test_fail "Build script missing Docker buildx check"
        return 1
    fi
    
    if grep -q "DOCKER_BUILDKIT=1" "$BUILD_SCRIPT"; then
        log_test_pass "Build script enables Docker BuildKit"
    else
        log_test_fail "Build script missing BuildKit enablement"
        return 1
    fi
    
    return 0
}

test_compose_file_selection() {
    log_test_start "Testing compose file selection logic"
    
    # Check that script contains logic to select correct compose file
    if grep -q "docker-compose.dev.yml" "$BUILD_SCRIPT"; then
        log_test_pass "Build script references dev compose file"
    else
        log_test_fail "Build script missing dev compose file reference"
        return 1
    fi
    
    if grep -q "docker-compose.yml" "$BUILD_SCRIPT"; then
        log_test_pass "Build script references prod compose file"
    else
        log_test_fail "Build script missing prod compose file reference"
        return 1
    fi
    
    # Check MODE variable usage
    if grep -q 'MODE=' "$BUILD_SCRIPT"; then
        log_test_pass "Build script uses MODE variable"
    else
        log_test_fail "Build script missing MODE variable"
        return 1
    fi
    
    return 0
}

test_build_output_validation() {
    log_test_start "Testing build output and logging"
    
    # Test that script has proper logging functions
    local logging_functions=(
        "print_status"
        "print_error"
        "print_warning"
    )
    
    for func in "${logging_functions[@]}"; do
        if grep -q "$func" "$BUILD_SCRIPT"; then
            log_info "✓ Found logging function: $func"
        else
            log_test_fail "Missing logging function: $func"
            return 1
        fi
    done
    
    # Test colored output support
    if grep -q "RED=" "$BUILD_SCRIPT" && grep -q "GREEN=" "$BUILD_SCRIPT"; then
        log_test_pass "Build script supports colored output"
    else
        log_test_fail "Build script missing colored output support"
        return 1
    fi
    
    return 0
}

test_cleanup_behavior() {
    log_test_start "Testing cleanup and error handling"
    
    # Check for proper error handling
    if grep -q "set -e" "$BUILD_SCRIPT"; then
        log_test_pass "Build script has error handling (set -e)"
    else
        log_test_fail "Build script missing error handling"
        return 1
    fi
    
    # Check for proper directory handling
    if grep -q "cd.*PROJECT_ROOT" "$BUILD_SCRIPT" || grep -q "SCRIPT_DIR" "$BUILD_SCRIPT"; then
        log_test_pass "Build script handles directory navigation"
    else
        log_test_fail "Build script missing directory handling"
        return 1
    fi
    
    return 0
}

# Run main function
main "$@" 