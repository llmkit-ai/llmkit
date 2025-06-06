#!/bin/bash

# Docker Build Smoke Tests
# Quick validation that Docker builds work correctly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

# Test configuration
SMOKE_TEST_TIMEOUT=${INDIVIDUAL_TEST_TIMEOUT:-300}  # Use main timeout or fallback to 5 minutes
CLEANUP_MODE=${1:-"auto"}  # Accept cleanup mode as first parameter

main() {
    log_info "Starting Docker Build Smoke Tests"
    log_info "Timeout: ${SMOKE_TEST_TIMEOUT}s"
    log_info "Cleanup mode: $CLEANUP_MODE"
    
    # Ensure we're in project root
    cd "$PROJECT_ROOT"
    
    # Conditional cleanup before starting
    if [[ "$CLEANUP_MODE" != "none" ]]; then
        log_info "Performing initial cleanup..."
        cleanup_all
    else
        log_info "Skipping initial cleanup (--no-cleanup mode)"
    fi
    
    # Run smoke tests
    test_build_script_validation
    test_compose_file_validation
    test_dev_build
    test_prod_build
    test_build_script_arguments
    
    print_test_summary
}

test_build_script_validation() {
    log_test_start "Build script validation smoke test"
    
    # Check if build script exists and is executable
    if ! validate_build_script; then
        return 1
    fi
    
    # Test help argument
    if ! $BUILD_SCRIPT --help >/dev/null 2>&1; then
        log_test_fail "Build script --help failed"
        return 1
    fi
    
    log_test_pass "Build script validation smoke test"
    return 0
}

test_compose_file_validation() {
    log_test_start "Compose file validation smoke test"
    
    if ! validate_compose_files; then
        return 1
    fi
    
    log_test_pass "Compose file validation smoke test"
    return 0
}

test_dev_build() {
    log_test_start "Development build smoke test"
    
    log_info "Building in development mode..."
    start_timer
    
    # Use timeout to prevent hanging
    if ! with_timeout $SMOKE_TEST_TIMEOUT $BUILD_SCRIPT --dev; then
        log_test_fail "Development build failed or timed out"
        return 1
    fi
    
    end_timer
    
    # Verify containers are running
    if ! check_containers_running "dev"; then
        log_test_fail "Development containers not running properly"
        return 1
    fi
    
    log_test_pass "Development build smoke test"
    return 0
}

test_prod_build() {
    log_test_start "Production build smoke test"
    
    # Cleanup dev containers first
    cleanup_containers "$COMPOSE_DEV_FILE"
    
    log_info "Building in production mode..."
    start_timer
    
    # Use timeout to prevent hanging
    if ! with_timeout $SMOKE_TEST_TIMEOUT $BUILD_SCRIPT --prod; then
        log_test_fail "Production build failed or timed out"
        return 1
    fi
    
    end_timer
    
    # Verify containers are running
    if ! check_containers_running "prod"; then
        log_test_fail "Production containers not running properly"
        return 1
    fi
    
    log_test_pass "Production build smoke test"
    return 0
}

test_build_script_arguments() {
    log_test_start "Build script arguments smoke test"
    
    # Test invalid argument
    if $BUILD_SCRIPT --invalid-arg >/dev/null 2>&1; then
        log_test_fail "Build script should fail with invalid argument"
        return 1
    fi
    
    # Test --rebuild flag (quick test)
    if ! with_timeout $SMOKE_TEST_TIMEOUT $BUILD_SCRIPT --dev --rebuild >/dev/null 2>&1; then
        log_test_fail "Build script --rebuild flag failed"
        return 1
    fi
    
    log_test_pass "Build script arguments smoke test"
    return 0
}

check_containers_running() {
    local mode=$1
    local compose_file
    
    case $mode in
        "dev")
            compose_file="$COMPOSE_DEV_FILE"
            ;;
        "prod")
            compose_file="$COMPOSE_PROD_FILE"
            ;;
        *)
            log_error "Invalid mode: $mode"
            return 1
            ;;
    esac
    
    # Check if containers exist and are running
    local services=$($DOCKER_COMPOSE_CMD -f "$compose_file" ps --services)
    for service in $services; do
        local container_id=$(get_container_id "$service" "$compose_file")
        if [[ -z "$container_id" ]]; then
            log_error "Container for service $service not found"
            return 1
        fi
        
        local status=$(get_container_status "$container_id")
        if [[ "$status" != "running" ]]; then
            log_error "Container $service is not running (status: $status)"
            return 1
        fi
        
        log_info "✓ Service $service is running"
    done
    
    return 0
}

# Conditional cleanup on exit
if [[ "$CLEANUP_MODE" != "none" ]]; then
    trap 'cleanup_all' EXIT
else
    log_info "No cleanup on exit (--no-cleanup mode)"
fi

# Run main function
main "$@" 