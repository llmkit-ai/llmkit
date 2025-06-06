#!/bin/bash

# Docker Runtime Smoke Tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

# Test configuration
RUNTIME_TEST_TIMEOUT=${INDIVIDUAL_TEST_TIMEOUT:-60}
HEALTH_CHECK_TIMEOUT=30
CLEANUP_MODE=${1:-"auto"}  # Accept cleanup mode as first parameter

main() {
    log_info "Starting Docker Runtime Smoke Tests"
    log_info "Runtime timeout: ${RUNTIME_TEST_TIMEOUT}s"
    log_info "Health check timeout: ${HEALTH_CHECK_TIMEOUT}s"
    log_info "Cleanup mode: $CLEANUP_MODE"
    
    # Ensure we're in project root
    cd "$PROJECT_ROOT"
    
    # Check if we have running containers to test
    if ! check_containers_exist; then
        log_error "No containers found. Run build tests first."
        exit 1
    fi
    
    # Run runtime tests
    test_service_ports
    test_container_health
    test_basic_connectivity
    test_volume_mounts
    test_environment_variables
    
    print_test_summary
}

check_containers_exist() {
    log_test_start "Checking for existing containers"
    
    # Check dev containers first, then prod
    local dev_containers=$($DOCKER_COMPOSE_CMD -f "$COMPOSE_DEV_FILE" ps -q 2>/dev/null | wc -l)
    local prod_containers=$($DOCKER_COMPOSE_CMD -f "$COMPOSE_PROD_FILE" ps -q 2>/dev/null | wc -l)
    
    if [[ $dev_containers -gt 0 ]]; then
        log_info "Found $dev_containers development containers"
        export CURRENT_MODE="dev"
        export CURRENT_COMPOSE_FILE="$COMPOSE_DEV_FILE"
        log_test_pass "Container existence check"
        return 0
    elif [[ $prod_containers -gt 0 ]]; then
        log_info "Found $prod_containers production containers"
        export CURRENT_MODE="prod"
        export CURRENT_COMPOSE_FILE="$COMPOSE_PROD_FILE"
        log_test_pass "Container existence check"
        return 0
    else
        log_test_fail "No containers found"
        return 1
    fi
}

test_service_ports() {
    log_test_start "Testing service port accessibility"
    
    local success=true
    
    # Load environment variables for port configuration
    if [[ -f "$PROJECT_ROOT/.env" ]]; then
        source "$PROJECT_ROOT/.env"
    fi
    
    local backend_port=${BACKEND_PORT:-8000}
    local ui_port=${UI_PORT:-3000}
    
    # Test backend port
    if ! test_service_connectivity "backend" "$backend_port" 30; then
        success=false
    fi
    
    # Test UI port
    if ! test_service_connectivity "ui" "$ui_port" 30; then
        success=false
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Service port accessibility"
        return 0
    else
        return 1
    fi
}

test_container_health() {
    log_test_start "Testing container health status"
    
    local success=true
    local services=$($DOCKER_COMPOSE_CMD -f "$CURRENT_COMPOSE_FILE" ps --services)
    
    for service in $services; do
        if ! check_container_health "$service" "$CURRENT_COMPOSE_FILE" 30; then
            success=false
        fi
    done
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Container health status"
        return 0
    else
        return 1
    fi
}

test_basic_connectivity() {
    log_test_start "Testing basic HTTP connectivity"
    
    # Load environment variables
    if [[ -f "$PROJECT_ROOT/.env" ]]; then
        source "$PROJECT_ROOT/.env"
    fi
    
    local backend_port=${BACKEND_PORT:-8000}
    local ui_port=${UI_PORT:-3000}
    local success=true
    
    # Test backend basic response
    log_info "Testing backend basic connectivity..."
    if ! curl -s -f "http://localhost:$backend_port" >/dev/null 2>&1; then
        # Try common endpoints that might exist
        if ! curl -s -f "http://localhost:$backend_port/health" >/dev/null 2>&1 && \
           ! curl -s -f "http://localhost:$backend_port/api" >/dev/null 2>&1 && \
           ! curl -s -f "http://localhost:$backend_port/" >/dev/null 2>&1; then
            log_test_fail "Backend not responding on port $backend_port"
            success=false
        else
            log_info "✓ Backend responding on port $backend_port"
        fi
    else
        log_info "✓ Backend responding on port $backend_port"
    fi
    
    # Test UI basic response
    log_info "Testing UI basic connectivity..."
    if ! curl -s -f "http://localhost:$ui_port" >/dev/null 2>&1; then
        log_test_fail "UI not responding on port $ui_port"
        success=false
    else
        log_info "✓ UI responding on port $ui_port"
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Basic HTTP connectivity"
        return 0
    else
        return 1
    fi
}

test_volume_mounts() {
    log_test_start "Testing volume mounts"
    
    local success=true
    local services=$($DOCKER_COMPOSE_CMD -f "$CURRENT_COMPOSE_FILE" ps --services)
    
    for service in $services; do
        if ! check_volume_mounts "$service" "$CURRENT_COMPOSE_FILE"; then
            success=false
        fi
    done
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Volume mounts verification"
        return 0
    else
        return 1
    fi
}

test_environment_variables() {
    log_test_start "Testing environment variables"
    
    local success=true
    
    # Expected environment variables for backend
    local backend_vars="RUST_LOG DATABASE_URL"
    if ! validate_env_vars "backend" "$backend_vars" "$CURRENT_COMPOSE_FILE"; then
        success=false
    fi
    
    # Check if UI container exists before testing
    local ui_container=$(get_container_id "ui" "$CURRENT_COMPOSE_FILE")
    if [[ -n "$ui_container" ]]; then
        # Expected environment variables for UI (more flexible as they might not all be set)
        local ui_vars="NODE_ENV"
        if ! validate_env_vars "ui" "$ui_vars" "$CURRENT_COMPOSE_FILE"; then
            # Don't fail the test if UI env vars are missing (they might be build-time)
            log_warning "Some UI environment variables missing (this might be normal)"
        fi
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Environment variables verification"
        return 0
    else
        return 1
    fi
}

# Run main function
main "$@" 