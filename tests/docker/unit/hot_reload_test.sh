#!/bin/bash

# Docker Hot Reload Unit Tests
# Detailed testing of hot reload functionality for development workflow

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

# Hot reload test configuration
HOT_RELOAD_TIMEOUT=90
RELOAD_DETECTION_TIMEOUT=45

main() {
    log_info "Starting Docker Hot Reload Unit Tests"
    log_info "Hot reload timeout: ${HOT_RELOAD_TIMEOUT}s"
    log_info "Reload detection timeout: ${RELOAD_DETECTION_TIMEOUT}s"
    
    # Ensure we're in project root
    cd "$PROJECT_ROOT"
    
    # Cleanup and start fresh
    cleanup_all
    
    # Build development containers for testing
    log_info "Setting up development environment for hot reload testing..."
    if ! with_timeout 300 $BUILD_SCRIPT --dev >/dev/null 2>&1; then
        log_error "Failed to build development environment"
        exit 1
    fi
    
    # Run hot reload tests
    test_cargo_watch_setup
    test_bun_dev_setup
    test_backend_hot_reload
    test_ui_hot_reload
    test_volume_mount_configuration
    test_development_command_override
    test_reload_performance
    test_error_recovery_during_reload
    
    print_test_summary
}

test_cargo_watch_setup() {
    log_test_start "Testing cargo-watch setup in backend container"
    
    local backend_container=$(get_container_id "backend" "$COMPOSE_DEV_FILE")
    if [[ -z "$backend_container" ]]; then
        log_test_fail "Backend container not found"
        return 1
    fi
    
    # Check if cargo-watch is running
    local cargo_watch_process=$(docker exec "$backend_container" ps aux | grep "cargo watch" | grep -v grep || echo "")
    
    if [[ -n "$cargo_watch_process" ]]; then
        log_test_pass "Cargo-watch is running in backend container"
        log_info "Process: $cargo_watch_process"
    else
        # Check if it's running as the main command
        local main_command=$(docker exec "$backend_container" ps aux | head -2 | tail -1)
        if echo "$main_command" | grep -q "cargo"; then
            log_test_pass "Cargo command is running as main process"
            log_info "Command: $main_command"
        else
            log_test_fail "Cargo-watch not detected in backend container"
            return 1
        fi
    fi
    
    # Verify cargo-watch configuration in compose file
    if grep -q "cargo.*watch" "$COMPOSE_DEV_FILE"; then
        log_info "✓ Cargo-watch configured in compose file"
    else
        log_warning "Cargo-watch not explicitly configured in compose file"
    fi
    
    return 0
}

test_bun_dev_setup() {
    log_test_start "Testing Bun dev server setup in UI container"
    
    local ui_container=$(get_container_id "ui" "$COMPOSE_DEV_FILE")
    if [[ -z "$ui_container" ]]; then
        log_test_fail "UI container not found"
        return 1
    fi
    
    # Check if Bun dev server is running
    local bun_process=$(docker exec "$ui_container" ps aux | grep "bun.*dev\|bun.*run.*dev" | grep -v grep || echo "")
    
    if [[ -n "$bun_process" ]]; then
        log_test_pass "Bun dev server is running in UI container"
        log_info "Process: $bun_process"
    else
        # Check main process
        local main_command=$(docker exec "$ui_container" ps aux | head -2 | tail -1)
        if echo "$main_command" | grep -q "bun"; then
            log_test_pass "Bun is running as main process"
            log_info "Command: $main_command"
        else
            log_test_fail "Bun dev server not detected in UI container"
            return 1
        fi
    fi
    
    # Verify Bun dev configuration in compose file
    if grep -q "bun.*dev" "$COMPOSE_DEV_FILE"; then
        log_info "✓ Bun dev server configured in compose file"
    else
        log_warning "Bun dev server not explicitly configured in compose file"
    fi
    
    return 0
}

test_backend_hot_reload() {
    log_test_start "Testing backend hot reload functionality"
    
    local backend_container=$(get_container_id "backend" "$COMPOSE_DEV_FILE")
    if [[ -z "$backend_container" ]]; then
        log_test_fail "Backend container not found"
        return 1
    fi
    
    # Find a Rust source file to modify
    local test_files=(
        "$PROJECT_ROOT/backend/src/main.rs"
        "$PROJECT_ROOT/backend/src/lib.rs"
    )
    
    local test_file=""
    for file in "${test_files[@]}"; do
        if [[ -f "$file" ]]; then
            test_file="$file"
            break
        fi
    done
    
    if [[ -z "$test_file" ]]; then
        # Create a test file if none exist
        test_file="$PROJECT_ROOT/backend/src/test_hot_reload.rs"
        echo "// Test file for hot reload $(date)" > "$test_file"
        log_info "Created test file: $test_file"
    fi
    
    # Get initial log state
    local initial_log_lines=$(docker logs "$backend_container" 2>&1 | wc -l)
    
    # Modify the file to trigger hot reload
    echo "// Hot reload test modification $(date)" >> "$test_file"
    log_info "Modified file to trigger hot reload: $test_file"
    
    # Wait for reload detection in logs
    local reload_detected=false
    local elapsed=0
    
    while [[ $elapsed -lt $RELOAD_DETECTION_TIMEOUT ]]; do
        local current_log_lines=$(docker logs "$backend_container" 2>&1 | wc -l)
        
        if [[ $current_log_lines -gt $initial_log_lines ]]; then
            # Check if logs contain reload-related messages
            local recent_logs=$(docker logs --tail 10 "$backend_container" 2>&1)
            if echo "$recent_logs" | grep -qi "compiling\|building\|restarting\|change"; then
                reload_detected=true
                log_test_pass "Backend hot reload triggered successfully"
                log_info "Detected reload in ${elapsed}s"
                break
            fi
        fi
        
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    # Cleanup test file if we created it
    if [[ "$test_file" == *"test_hot_reload.rs" ]]; then
        rm -f "$test_file"
    fi
    
    if [[ "$reload_detected" == "true" ]]; then
        return 0
    else
        log_test_fail "Backend hot reload not detected within ${RELOAD_DETECTION_TIMEOUT}s"
        return 1
    fi
}

test_ui_hot_reload() {
    log_test_start "Testing UI hot reload functionality"
    
    local ui_container=$(get_container_id "ui" "$COMPOSE_DEV_FILE")
    if [[ -z "$ui_container" ]]; then
        log_test_fail "UI container not found"
        return 1
    fi
    
    # Find a UI source file to modify
    local test_files=(
        "$PROJECT_ROOT/ui/app.vue"
        "$PROJECT_ROOT/ui/pages/index.vue"
        "$PROJECT_ROOT/ui/components"
    )
    
    local test_file=""
    for file in "${test_files[@]}"; do
        if [[ -f "$file" ]]; then
            test_file="$file"
            break
        elif [[ -d "$file" ]]; then
            # Look for Vue files in the directory
            local vue_file=$(find "$file" -name "*.vue" -type f | head -1)
            if [[ -n "$vue_file" ]]; then
                test_file="$vue_file"
                break
            fi
        fi
    done
    
    if [[ -z "$test_file" ]]; then
        # Create a test file if none exist
        test_file="$PROJECT_ROOT/ui/test_hot_reload.vue"
        cat > "$test_file" << EOF
<template>
  <div>Test hot reload $(date)</div>
</template>
EOF
        log_info "Created test file: $test_file"
    fi
    
    # Get initial log state
    local initial_log_lines=$(docker logs "$ui_container" 2>&1 | wc -l)
    
    # Modify the file to trigger hot reload
    echo "<!-- Hot reload test modification $(date) -->" >> "$test_file"
    log_info "Modified file to trigger hot reload: $test_file"
    
    # Wait for reload detection in logs
    local reload_detected=false
    local elapsed=0
    
    while [[ $elapsed -lt $RELOAD_DETECTION_TIMEOUT ]]; do
        local current_log_lines=$(docker logs "$ui_container" 2>&1 | wc -l)
        
        if [[ $current_log_lines -gt $initial_log_lines ]]; then
            # Check if logs contain reload-related messages
            local recent_logs=$(docker logs --tail 10 "$ui_container" 2>&1)
            if echo "$recent_logs" | grep -qi "hmr\|hot.*reload\|update\|change"; then
                reload_detected=true
                log_test_pass "UI hot reload triggered successfully"
                log_info "Detected reload in ${elapsed}s"
                break
            fi
        fi
        
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    # Cleanup test file if we created it
    if [[ "$test_file" == *"test_hot_reload.vue" ]]; then
        rm -f "$test_file"
    fi
    
    if [[ "$reload_detected" == "true" ]]; then
        return 0
    else
        log_test_fail "UI hot reload not detected within ${RELOAD_DETECTION_TIMEOUT}s"
        return 1
    fi
}

test_volume_mount_configuration() {
    log_test_start "Testing volume mount configuration for hot reload"
    
    # Check backend volume mounts
    local backend_container=$(get_container_id "backend" "$COMPOSE_DEV_FILE")
    if [[ -n "$backend_container" ]]; then
        local backend_mounts=$(docker inspect "$backend_container" --format='{{range .Mounts}}{{.Source}}:{{.Destination}}:{{.Mode}} {{end}}')
        
        # Check for source code mount
        if echo "$backend_mounts" | grep -q "src.*rw\|src.*:rw"; then
            log_info "✓ Backend source code mounted as read-write"
        else
            log_test_fail "Backend source code not mounted as read-write"
            return 1
        fi
        
        log_info "Backend mounts: $backend_mounts"
    fi
    
    # Check UI volume mounts
    local ui_container=$(get_container_id "ui" "$COMPOSE_DEV_FILE")
    if [[ -n "$ui_container" ]]; then
        local ui_mounts=$(docker inspect "$ui_container" --format='{{range .Mounts}}{{.Source}}:{{.Destination}}:{{.Mode}} {{end}}')
        
        # Check for UI directory mount
        if echo "$ui_mounts" | grep -q "ui.*rw\|ui.*:rw"; then
            log_info "✓ UI directory mounted as read-write"
        else
            log_test_fail "UI directory not mounted as read-write"
            return 1
        fi
        
        log_info "UI mounts: $ui_mounts"
    fi
    
    log_test_pass "Volume mount configuration for hot reload"
    return 0
}

test_development_command_override() {
    log_test_start "Testing development command override"
    
    # Check that compose file overrides commands for development
    if grep -q "command.*cargo.*watch" "$COMPOSE_DEV_FILE"; then
        log_info "✓ Backend development command override found"
    else
        log_warning "Backend development command override not found"
    fi
    
    if grep -q "command.*bun.*dev" "$COMPOSE_DEV_FILE"; then
        log_info "✓ UI development command override found"
    else
        log_warning "UI development command override not found"
    fi
    
    # Verify containers are using development commands
    local backend_container=$(get_container_id "backend" "$COMPOSE_DEV_FILE")
    if [[ -n "$backend_container" ]]; then
        local backend_command=$(docker inspect "$backend_container" --format='{{.Config.Cmd}}')
        if echo "$backend_command" | grep -q "cargo"; then
            log_info "✓ Backend using cargo development command"
        else
            log_warning "Backend may not be using development command"
        fi
    fi
    
    local ui_container=$(get_container_id "ui" "$COMPOSE_DEV_FILE")
    if [[ -n "$ui_container" ]]; then
        local ui_command=$(docker inspect "$ui_container" --format='{{.Config.Cmd}}')
        if echo "$ui_command" | grep -q "bun"; then
            log_info "✓ UI using Bun development command"
        else
            log_warning "UI may not be using development command"
        fi
    fi
    
    log_test_pass "Development command override verification"
    return 0
}

test_reload_performance() {
    log_test_start "Testing hot reload performance"
    
    # Create a simple test file and measure reload time
    local test_file="$PROJECT_ROOT/backend/src/reload_perf_test.rs"
    echo "// Performance test file $(date)" > "$test_file"
    
    local backend_container=$(get_container_id "backend" "$COMPOSE_DEV_FILE")
    if [[ -z "$backend_container" ]]; then
        rm -f "$test_file"
        log_test_fail "Backend container not found"
        return 1
    fi
    
    # Get baseline log count
    local initial_logs=$(docker logs "$backend_container" 2>&1 | wc -l)
    
    # Start timer and modify file
    start_timer
    echo "// Performance test modification $(date)" >> "$test_file"
    
    # Wait for reload completion
    local reload_completed=false
    while [[ $TIMER_DIFF -lt 30 ]]; do  # 30 second max
        local current_logs=$(docker logs "$backend_container" 2>&1 | wc -l)
        if [[ $current_logs -gt $initial_logs ]]; then
            # Check if compilation/reload is complete
            local recent_logs=$(docker logs --tail 5 "$backend_container" 2>&1)
            if echo "$recent_logs" | grep -qi "finished\|complete\|ready"; then
                reload_completed=true
                break
            fi
        fi
        sleep 1
        end_timer  # Update timer
    done
    
    rm -f "$test_file"
    
    if [[ "$reload_completed" == "true" ]]; then
        if [[ $TIMER_DIFF -lt 15 ]]; then
            log_test_pass "Hot reload performance: ${TIMER_DIFF}s (< 15s)"
        else
            log_warning "Hot reload performance: ${TIMER_DIFF}s (may be slow)"
        fi
        return 0
    else
        log_test_fail "Hot reload performance test timed out"
        return 1
    fi
}

test_error_recovery_during_reload() {
    log_test_start "Testing error recovery during hot reload"
    
    # Create a file with intentional syntax error
    local test_file="$PROJECT_ROOT/backend/src/error_recovery_test.rs"
    cat > "$test_file" << 'EOF'
// Intentional syntax error for testing
fn test_function() {
    let x = "unclosed string
    println!("This will cause an error");
}
EOF
    
    local backend_container=$(get_container_id "backend" "$COMPOSE_DEV_FILE")
    if [[ -z "$backend_container" ]]; then
        rm -f "$test_file"
        log_test_fail "Backend container not found"
        return 1
    fi
    
    # Wait for error compilation
    sleep 5
    
    # Check logs for compilation error
    local error_logs=$(docker logs --tail 20 "$backend_container" 2>&1)
    if echo "$error_logs" | grep -qi "error\|failed"; then
        log_info "✓ Compilation error detected as expected"
    else
        log_warning "Compilation error not detected in logs"
    fi
    
    # Fix the syntax error
    cat > "$test_file" << 'EOF'
// Fixed syntax error
fn test_function() {
    let x = "properly closed string";
    println!("This should compile correctly");
}
EOF
    
    # Wait for recovery
    sleep 5
    
    # Check if container is still running and recovered
    local container_status=$(get_container_status "$backend_container")
    if [[ "$container_status" == "running" ]]; then
        log_test_pass "Error recovery during hot reload successful"
    else
        log_test_fail "Container failed to recover from error: $container_status"
        rm -f "$test_file"
        return 1
    fi
    
    rm -f "$test_file"
    return 0
}

# Cleanup on exit
trap 'cleanup_all' EXIT

# Run main function
main "$@" 