#!/bin/bash

# Docker-specific Helper Functions

source "$(dirname "${BASH_SOURCE[0]}")/test_helpers.sh"

# Environment setup
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../" && pwd)"
COMPOSE_DEV_FILE="$PROJECT_ROOT/docker/docker-compose.dev.yml"
COMPOSE_PROD_FILE="$PROJECT_ROOT/docker/docker-compose.yml"
BUILD_SCRIPT="$PROJECT_ROOT/docker/build.sh"

# Docker Compose version detection (same logic as build.sh)
DOCKER_COMPOSE_CMD="docker compose"
if ! docker compose version >/dev/null 2>&1; then
    DOCKER_COMPOSE_CMD="docker-compose"
fi

# Build validation
validate_build_script() {
    log_test_start "Validating build script exists and is executable"
    
    if [[ ! -f "$BUILD_SCRIPT" ]]; then
        log_test_fail "Build script not found: $BUILD_SCRIPT"
        return 1
    fi
    
    if [[ ! -x "$BUILD_SCRIPT" ]]; then
        log_test_fail "Build script is not executable: $BUILD_SCRIPT"
        return 1
    fi
    
    log_test_pass "Build script validation"
    return 0
}

validate_compose_files() {
    log_test_start "Validating Docker compose files"
    
    local success=true
    
    if [[ ! -f "$COMPOSE_DEV_FILE" ]]; then
        log_test_fail "Dev compose file not found: $COMPOSE_DEV_FILE"
        success=false
    fi
    
    if [[ ! -f "$COMPOSE_PROD_FILE" ]]; then
        log_test_fail "Prod compose file not found: $COMPOSE_PROD_FILE"
        success=false
    fi
    
    # Validate compose file syntax
    if ! $DOCKER_COMPOSE_CMD -f "$COMPOSE_DEV_FILE" config >/dev/null 2>&1; then
        log_test_fail "Dev compose file syntax invalid"
        success=false
    fi
    
    if ! $DOCKER_COMPOSE_CMD -f "$COMPOSE_PROD_FILE" config >/dev/null 2>&1; then
        log_test_fail "Prod compose file syntax invalid"
        success=false
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Docker compose files validation"
        return 0
    else
        return 1
    fi
}

# Build performance monitoring
measure_build_time() {
    local mode=$1
    local rebuild_flag=${2:-""}
    
    log_info "Measuring build time for $mode mode${rebuild_flag:+ with $rebuild_flag}"
    
    local build_args=""
    case $mode in
        "dev")
            build_args="--dev"
            ;;
        "prod")
            build_args="--prod"
            ;;
        *)
            log_error "Invalid mode: $mode"
            return 1
            ;;
    esac
    
    if [[ -n "$rebuild_flag" ]]; then
        build_args="$build_args $rebuild_flag"
    fi
    
    start_timer
    if ! $BUILD_SCRIPT $build_args >/dev/null 2>&1; then
        log_error "Build failed for $mode mode"
        return 1
    fi
    end_timer
    
    return 0
}

# Cache validation
check_docker_cache() {
    log_test_start "Checking Docker cache effectiveness"
    
    # Build once to populate cache
    log_info "Initial build to populate cache..."
    if ! $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Initial build failed"
        return 1
    fi
    
    # Measure cached build time
    log_info "Measuring cached build time..."
    start_timer
    if ! $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Cached build failed"
        return 1
    fi
    local cached_time=$TIMER_DIFF
    end_timer
    
    # Verify cached build is faster than threshold (30 seconds)
    if [[ $cached_time -gt 30 ]]; then
        log_test_fail "Cached build too slow: ${cached_time}s (expected < 30s)"
        return 1
    fi
    
    log_test_pass "Docker cache effectiveness (${cached_time}s)"
    return 0
}

# Container health checks
check_container_health() {
    local service_name=$1
    local compose_file=${2:-"$COMPOSE_DEV_FILE"}
    local timeout=${3:-60}
    
    log_test_start "Checking health of $service_name container"
    
    local container_id=$(get_container_id "$service_name" "$compose_file")
    if [[ -z "$container_id" ]]; then
        log_test_fail "Container $service_name not found"
        return 1
    fi
    
    local elapsed=0
    while [[ $elapsed -lt $timeout ]]; do
        local status=$(get_container_status "$container_id")
        
        case $status in
            "running")
                log_test_pass "Container $service_name is healthy"
                return 0
                ;;
            "exited"|"dead")
                log_test_fail "Container $service_name failed (status: $status)"
                return 1
                ;;
            *)
                log_info "Container $service_name status: $status (waiting...)"
                ;;
        esac
        
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    log_test_fail "Container $service_name health check timeout"
    return 1
}

# Service connectivity tests
test_service_connectivity() {
    local service_name=$1
    local port=$2
    local timeout=${3:-30}
    
    log_test_start "Testing connectivity to $service_name:$port"
    
    if wait_for_service "localhost" "$port" "$timeout"; then
        log_test_pass "Service $service_name connectivity"
        return 0
    else
        log_test_fail "Service $service_name connectivity"
        return 1
    fi
}

# Volume verification
check_volume_mounts() {
    local service_name=$1
    local compose_file=${2:-"$COMPOSE_DEV_FILE"}
    
    log_test_start "Checking volume mounts for $service_name"
    
    local container_id=$(get_container_id "$service_name" "$compose_file")
    if [[ -z "$container_id" ]]; then
        log_test_fail "Container $service_name not found"
        return 1
    fi
    
    # Get mount information
    local mounts=$(docker inspect "$container_id" --format='{{range .Mounts}}{{.Source}}:{{.Destination}} {{end}}')
    
    if [[ -z "$mounts" ]]; then
        log_test_fail "No volume mounts found for $service_name"
        return 1
    fi
    
    log_test_pass "Volume mounts verified for $service_name"
    log_info "Mounts: $mounts"
    return 0
}

# Hot reload testing
test_hot_reload() {
    local service_name=$1
    local test_file_path=$2
    local compose_file=${3:-"$COMPOSE_DEV_FILE"}
    
    log_test_start "Testing hot reload for $service_name"
    
    if [[ ! -f "$test_file_path" ]]; then
        log_test_fail "Test file not found: $test_file_path"
        return 1
    fi
    
    # Get initial container logs length
    local container_id=$(get_container_id "$service_name" "$compose_file")
    local initial_logs=$(docker logs "$container_id" 2>&1 | wc -l)
    
    # Touch the test file to trigger rebuild
    touch "$test_file_path"
    
    # Wait for rebuild indication in logs
    local elapsed=0
    local timeout=30
    while [[ $elapsed -lt $timeout ]]; do
        local current_logs=$(docker logs "$container_id" 2>&1 | wc -l)
        if [[ $current_logs -gt $initial_logs ]]; then
            log_test_pass "Hot reload triggered for $service_name"
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    log_test_fail "Hot reload not detected for $service_name"
    return 1
}

# Environment variable validation
validate_env_vars() {
    local service_name=$1
    local expected_vars=$2
    local compose_file=${3:-"$COMPOSE_DEV_FILE"}
    
    log_test_start "Validating environment variables for $service_name"
    
    local container_id=$(get_container_id "$service_name" "$compose_file")
    if [[ -z "$container_id" ]]; then
        log_test_fail "Container $service_name not found"
        return 1
    fi
    
    local success=true
    for var in $expected_vars; do
        local value=$(docker exec "$container_id" printenv "$var" 2>/dev/null)
        if [[ -z "$value" ]]; then
            log_test_fail "Environment variable $var not set in $service_name"
            success=false
        else
            log_info "✓ $var is set in $service_name"
        fi
    done
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Environment variables validation for $service_name"
        return 0
    else
        return 1
    fi
}

# Docker image analysis
analyze_image_size() {
    local image_name=$1
    
    log_test_start "Analyzing image size for $image_name"
    
    if ! docker image inspect "$image_name" >/dev/null 2>&1; then
        log_test_fail "Image $image_name not found"
        return 1
    fi
    
    local size=$(docker image inspect "$image_name" --format='{{.Size}}')
    local size_mb=$((size / 1024 / 1024))
    
    log_test_pass "Image $image_name size: ${size_mb}MB"
    
    # Alert if image is unusually large (> 1GB)
    if [[ $size_mb -gt 1024 ]]; then
        log_warning "Image $image_name is large: ${size_mb}MB"
    fi
    
    return 0
}

# Cleanup with confirmation
safe_cleanup() {
    local force=${1:-false}
    
    if [[ "$force" != "true" ]]; then
        log_warning "This will remove all containers and images. Continue? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            log_info "Cleanup cancelled"
            return 0
        fi
    fi
    
    cleanup_all
    log_success "Cleanup completed"
} 