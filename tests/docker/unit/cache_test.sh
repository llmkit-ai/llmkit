#!/bin/bash

# Docker Cache Unit Tests
# Detailed testing of cache effectiveness and optimization features

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

# Cache test configuration
CACHE_TEST_TIMEOUT=600  # 10 minutes for comprehensive cache tests
CACHE_THRESHOLD_SECONDS=60  # Cached builds should be under 60s

main() {
    log_info "Starting Docker Cache Unit Tests"
    log_info "Cache test timeout: ${CACHE_TEST_TIMEOUT}s"
    log_info "Cache threshold: ${CACHE_THRESHOLD_SECONDS}s"
    
    # Ensure we're in project root
    cd "$PROJECT_ROOT"
    
    # Initial cleanup
    cleanup_all
    
    # Run cache tests
    test_dockerfile_cache_optimization
    test_cargo_chef_implementation
    test_bun_cache_implementation
    test_build_cache_effectiveness
    test_layer_caching
    test_volume_cache_persistence
    test_buildkit_cache_mounts
    test_cache_invalidation
    
    print_test_summary
}

test_dockerfile_cache_optimization() {
    log_test_start "Testing Dockerfile cache optimization structure"
    
    local success=true
    
    # Test backend Dockerfile structure
    local backend_dockerfile="$PROJECT_ROOT/backend/Dockerfile"
    if [[ -f "$backend_dockerfile" ]]; then
        # Check for cargo-chef usage
        if grep -q "cargo-chef" "$backend_dockerfile"; then
            log_info "✓ Backend Dockerfile uses cargo-chef"
        else
            log_test_fail "Backend Dockerfile missing cargo-chef optimization"
            success=false
        fi
        
        # Check for multi-stage build
        if grep -q "FROM.*AS" "$backend_dockerfile"; then
            log_info "✓ Backend Dockerfile uses multi-stage build"
        else
            log_test_fail "Backend Dockerfile missing multi-stage build"
            success=false
        fi
        
        # Check for dependency layer separation
        if grep -q "COPY.*Cargo.toml" "$backend_dockerfile"; then
            log_info "✓ Backend Dockerfile separates dependency copying"
        else
            log_warning "Backend Dockerfile may not separate dependencies optimally"
        fi
    else
        log_test_fail "Backend Dockerfile not found"
        success=false
    fi
    
    # Test UI Dockerfile structure
    local ui_dockerfile="$PROJECT_ROOT/ui/Dockerfile"
    if [[ -f "$ui_dockerfile" ]]; then
        # Check for Bun cache optimization
        if grep -q "bun" "$ui_dockerfile"; then
            log_info "✓ UI Dockerfile uses Bun"
        else
            log_warning "UI Dockerfile may not use Bun optimization"
        fi
        
        # Check for package.json copying before source
        if grep -q "COPY.*package.json" "$ui_dockerfile"; then
            log_info "✓ UI Dockerfile separates package.json copying"
        else
            log_warning "UI Dockerfile may not separate dependencies optimally"
        fi
    else
        log_test_fail "UI Dockerfile not found"
        success=false
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Dockerfile cache optimization structure"
        return 0
    else
        return 1
    fi
}

test_cargo_chef_implementation() {
    log_test_start "Testing cargo-chef implementation"
    
    local backend_dockerfile="$PROJECT_ROOT/backend/Dockerfile"
    local deps_dockerfile="$PROJECT_ROOT/backend/Dockerfile.deps"
    local success=true
    
    # Check for cargo-chef in main Dockerfile
    if [[ -f "$backend_dockerfile" ]]; then
        if grep -q "cargo-chef prepare" "$backend_dockerfile"; then
            log_info "✓ Found cargo-chef prepare step"
        else
            log_test_fail "Missing cargo-chef prepare step"
            success=false
        fi
        
        if grep -q "cargo-chef cook" "$backend_dockerfile"; then
            log_info "✓ Found cargo-chef cook step"
        else
            log_test_fail "Missing cargo-chef cook step"
            success=false
        fi
    fi
    
    # Check dependencies Dockerfile
    if [[ -f "$deps_dockerfile" ]]; then
        log_info "✓ Separate dependencies Dockerfile exists"
        
        if grep -q "cargo" "$deps_dockerfile"; then
            log_info "✓ Dependencies Dockerfile contains cargo commands"
        else
            log_warning "Dependencies Dockerfile may not contain cargo optimization"
        fi
    else
        log_warning "No separate dependencies Dockerfile found"
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Cargo-chef implementation"
        return 0
    else
        return 1
    fi
}

test_bun_cache_implementation() {
    log_test_start "Testing Bun cache implementation"
    
    local ui_dockerfile="$PROJECT_ROOT/ui/Dockerfile"
    local compose_dev="$PROJECT_ROOT/docker/docker-compose.dev.yml"
    local success=true
    
    # Check Dockerfile for Bun optimization
    if [[ -f "$ui_dockerfile" ]]; then
        if grep -q "bun install" "$ui_dockerfile"; then
            log_info "✓ UI Dockerfile uses bun install"
        else
            log_warning "UI Dockerfile may not use Bun optimization"
        fi
    fi
    
    # Check docker-compose for Bun cache volumes
    if [[ -f "$compose_dev" ]]; then
        if grep -q "bun_cache" "$compose_dev"; then
            log_info "✓ Development compose has Bun cache volume"
        else
            log_warning "Development compose missing Bun cache volume"
            success=false
        fi
        
        if grep -q "node_modules" "$compose_dev"; then
            log_info "✓ Development compose has node_modules volume"
        else
            log_warning "Development compose missing node_modules volume"
        fi
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Bun cache implementation"
        return 0
    else
        return 1
    fi
}

test_build_cache_effectiveness() {
    log_test_start "Testing build cache effectiveness"
    
    # First build (cold cache)
    log_info "Performing initial build to populate cache..."
    start_timer
    if ! with_timeout $CACHE_TEST_TIMEOUT $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Initial build failed or timed out"
        return 1
    fi
    local cold_build_time=$TIMER_DIFF
    end_timer
    log_info "Cold build time: ${cold_build_time}s"
    
    # Second build (warm cache)
    log_info "Testing cached build performance..."
    start_timer
    if ! with_timeout $CACHE_TEST_TIMEOUT $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Cached build failed or timed out"
        return 1
    fi
    local warm_build_time=$TIMER_DIFF
    end_timer
    log_info "Warm build time: ${warm_build_time}s"
    
    # Verify cache effectiveness
    if [[ $warm_build_time -lt $CACHE_THRESHOLD_SECONDS ]]; then
        log_test_pass "Cache effectiveness: ${warm_build_time}s < ${CACHE_THRESHOLD_SECONDS}s threshold"
        
        # Calculate improvement
        local improvement=$((cold_build_time - warm_build_time))
        local improvement_percent=$((improvement * 100 / cold_build_time))
        log_info "Cache improvement: ${improvement}s (${improvement_percent}%)"
        
        return 0
    else
        log_test_fail "Cache not effective: ${warm_build_time}s >= ${CACHE_THRESHOLD_SECONDS}s threshold"
        return 1
    fi
}

test_layer_caching() {
    log_test_start "Testing Docker layer caching"
    
    # Build and get image layers
    if ! $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Build failed for layer testing"
        return 1
    fi
    
    # Get images created by the build
    local images=$(docker images --format "{{.Repository}}:{{.Tag}}" | grep -E "(backend|ui)" | head -5)
    
    if [[ -z "$images" ]]; then
        log_test_fail "No backend/ui images found for layer testing"
        return 1
    fi
    
    local success=true
    for image in $images; do
        # Get layer information
        local layers=$(docker history "$image" --format "{{.Size}}" | wc -l)
        
        if [[ $layers -gt 3 ]]; then
            log_info "✓ Image $image has $layers layers (good for caching)"
        else
            log_warning "Image $image has only $layers layers (may not be optimized)"
        fi
        
        # Check for zero-byte layers (cached layers)
        local zero_layers=$(docker history "$image" --format "{{.Size}}" | grep -c "0B" || echo "0")
        if [[ $zero_layers -gt 0 ]]; then
            log_info "✓ Image $image has $zero_layers cached layers"
        fi
    done
    
    log_test_pass "Docker layer caching analysis"
    return 0
}

test_volume_cache_persistence() {
    log_test_start "Testing volume cache persistence"
    
    # Start containers to create volumes
    if ! $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Build failed for volume testing"
        return 1
    fi
    
    # Check for expected cache volumes
    local expected_volumes=(
        "backend_target"
        "backend_deps"
        "bun_cache"
        "ui_node_modules"
    )
    
    local success=true
    for volume in "${expected_volumes[@]}"; do
        # Check if volume exists (it may have a prefix)
        local volume_exists=$(docker volume ls --format "{{.Name}}" | grep -c "$volume" || echo "0")
        
        if [[ $volume_exists -gt 0 ]]; then
            log_info "✓ Volume $volume exists"
        else
            log_warning "Volume $volume not found (may not be created yet)"
        fi
    done
    
    # Stop containers but keep volumes
    $DOCKER_COMPOSE_CMD -f "$COMPOSE_DEV_FILE" down >/dev/null 2>&1 || true
    
    # Verify volumes persist after container stop
    for volume in "${expected_volumes[@]}"; do
        local volume_exists=$(docker volume ls --format "{{.Name}}" | grep -c "$volume" || echo "0")
        
        if [[ $volume_exists -gt 0 ]]; then
            log_info "✓ Volume $volume persisted after container stop"
        else
            log_info "Volume $volume not found (may be expected)"
        fi
    done
    
    log_test_pass "Volume cache persistence"
    return 0
}

test_buildkit_cache_mounts() {
    log_test_start "Testing BuildKit cache mounts"
    
    local backend_dockerfile="$PROJECT_ROOT/backend/Dockerfile"
    local success=true
    
    # Check for BuildKit cache mount syntax in Dockerfiles
    if [[ -f "$backend_dockerfile" ]]; then
        if grep -q "mount=type=cache" "$backend_dockerfile"; then
            log_info "✓ Backend Dockerfile uses BuildKit cache mounts"
        else
            log_warning "Backend Dockerfile may not use BuildKit cache mounts"
        fi
        
        # Check for specific cache targets
        if grep -q "target=/.*cargo" "$backend_dockerfile"; then
            log_info "✓ Backend Dockerfile caches cargo directory"
        else
            log_warning "Backend Dockerfile may not cache cargo optimally"
        fi
    fi
    
    # Check if build script enables BuildKit
    if grep -q "DOCKER_BUILDKIT=1" "$BUILD_SCRIPT"; then
        log_info "✓ Build script enables Docker BuildKit"
    else
        log_test_fail "Build script missing BuildKit enablement"
        success=false
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "BuildKit cache mounts"
        return 0
    else
        return 1
    fi
}

test_cache_invalidation() {
    log_test_start "Testing cache invalidation behavior"
    
    # Build initial version
    if ! $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Initial build failed for cache invalidation test"
        return 1
    fi
    
    # Create a temporary source file change to test invalidation
    local test_file="$PROJECT_ROOT/backend/src/test_cache_invalidation.rs"
    echo "// Test cache invalidation $(date)" > "$test_file"
    
    # Build again and measure time
    start_timer
    if ! $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_test_fail "Rebuild failed after source change"
        rm -f "$test_file"
        return 1
    fi
    local rebuild_time=$TIMER_DIFF
    end_timer
    
    # Cleanup test file
    rm -f "$test_file"
    
    # Rebuild time should be reasonable (cache still mostly effective)
    if [[ $rebuild_time -lt 120 ]]; then  # 2 minutes threshold
        log_test_pass "Cache invalidation: rebuild time ${rebuild_time}s (< 120s)"
        return 0
    else
        log_test_fail "Cache invalidation: rebuild too slow ${rebuild_time}s"
        return 1
    fi
}

# Cleanup on exit
trap 'cleanup_all' EXIT

# Run main function
main "$@" 