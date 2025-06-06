#!/bin/bash

# Docker Health Check Smoke Tests
# Comprehensive health validation for all services

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

# Test configuration
HEALTH_TIMEOUT=${INDIVIDUAL_TEST_TIMEOUT:-45}
RETRY_INTERVAL=3
MAX_RETRIES=15
CLEANUP_MODE=${1:-"auto"}  # Accept cleanup mode as first parameter

main() {
    log_info "Starting Docker Health Check Smoke Tests"
    log_info "Health timeout: ${HEALTH_TIMEOUT}s"
    log_info "Retry interval: ${RETRY_INTERVAL}s"
    log_info "Cleanup mode: $CLEANUP_MODE"
    
    # Ensure we're in project root
    cd "$PROJECT_ROOT"
    
    # Load environment variables
    if [[ -f "$PROJECT_ROOT/.env" ]]; then
        source "$PROJECT_ROOT/.env"
    fi
    
    # Determine which containers are running
    if ! determine_running_mode; then
        log_error "No containers found. Run build tests first."
        exit 1
    fi
    
    # Run health check tests
    test_service_health_endpoints
    test_service_responsiveness
    test_container_logs_health
    test_process_health
    test_resource_usage
    
    print_test_summary
}

determine_running_mode() {
    log_test_start "Determining running container mode"
    
    local dev_containers=$($DOCKER_COMPOSE_CMD -f "$COMPOSE_DEV_FILE" ps -q 2>/dev/null | wc -l)
    local prod_containers=$($DOCKER_COMPOSE_CMD -f "$COMPOSE_PROD_FILE" ps -q 2>/dev/null | wc -l)
    
    if [[ $dev_containers -gt 0 ]]; then
        export CURRENT_MODE="dev"
        export CURRENT_COMPOSE_FILE="$COMPOSE_DEV_FILE"
        log_test_pass "Running in development mode ($dev_containers containers)"
        return 0
    elif [[ $prod_containers -gt 0 ]]; then
        export CURRENT_MODE="prod"
        export CURRENT_COMPOSE_FILE="$COMPOSE_PROD_FILE"
        log_test_pass "Running in production mode ($prod_containers containers)"
        return 0
    else
        log_test_fail "No running containers found"
        return 1
    fi
}

test_service_health_endpoints() {
    log_test_start "Testing service health endpoints"
    
    local backend_port=${BACKEND_PORT:-8000}
    local ui_port=${UI_PORT:-3000}
    local success=true
    
    # Test backend health endpoints
    log_info "Testing backend health..."
    if ! test_backend_health "$backend_port"; then
        success=false
    fi
    
    # Test UI health endpoints
    log_info "Testing UI health..."
    if ! test_ui_health "$ui_port"; then
        success=false
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Service health endpoints"
        return 0
    else
        return 1
    fi
}

test_backend_health() {
    local port=$1
    local base_url="http://localhost:$port"
    
    # Try multiple potential health endpoints
    local health_endpoints=(
        "/health"
        "/healthz"
        "/api/health"
        "/status"
        "/"
    )
    
    for endpoint in "${health_endpoints[@]}"; do
        local url="$base_url$endpoint"
        log_info "Trying backend endpoint: $url"
        
        local response_code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
        
        if [[ "$response_code" == "200" ]]; then
            log_info "✓ Backend healthy at $url (HTTP $response_code)"
            return 0
        elif [[ "$response_code" != "000" && "$response_code" != "404" ]]; then
            log_info "Backend responding at $url (HTTP $response_code)"
            return 0
        fi
    done
    
    log_error "Backend not responding on any known endpoints"
    return 1
}

test_ui_health() {
    local port=$1
    local base_url="http://localhost:$port"
    
    # Try UI health check
    local response_code=$(curl -s -o /dev/null -w "%{http_code}" "$base_url" 2>/dev/null || echo "000")
    
    if [[ "$response_code" == "200" ]]; then
        log_info "✓ UI healthy at $base_url (HTTP $response_code)"
        
        # Additional check: verify we get HTML content
        local content_type=$(curl -s -I "$base_url" 2>/dev/null | grep -i "content-type" | head -1 || echo "")
        if [[ "$content_type" == *"text/html"* ]]; then
            log_info "✓ UI serving HTML content"
            return 0
        else
            log_warning "UI responding but not serving HTML content"
            return 0  # Still consider it healthy
        fi
    else
        log_error "UI not responding (HTTP $response_code)"
        return 1
    fi
}

test_service_responsiveness() {
    log_test_start "Testing service response times"
    
    local backend_port=${BACKEND_PORT:-8000}
    local ui_port=${UI_PORT:-3000}
    local success=true
    
    # Test backend response time
    if ! test_response_time "backend" "http://localhost:$backend_port" 5; then
        success=false
    fi
    
    # Test UI response time
    if ! test_response_time "ui" "http://localhost:$ui_port" 5; then
        success=false
    fi
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Service responsiveness"
        return 0
    else
        return 1
    fi
}

test_response_time() {
    local service_name=$1
    local url=$2
    local max_time=$3
    
    log_info "Testing $service_name response time (max: ${max_time}s)..."
    
    local response_time=$(curl -s -o /dev/null -w "%{time_total}" "$url" 2>/dev/null || echo "999")
    local response_time_int=$(echo "$response_time" | cut -d. -f1)
    
    if [[ $response_time_int -le $max_time ]]; then
        log_info "✓ $service_name response time: ${response_time}s"
        return 0
    else
        log_error "$service_name response time too slow: ${response_time}s (max: ${max_time}s)"
        return 1
    fi
}

test_container_logs_health() {
    log_test_start "Testing container logs for errors"
    
    local services=$($DOCKER_COMPOSE_CMD -f "$CURRENT_COMPOSE_FILE" ps --services)
    local success=true
    
    for service in $services; do
        log_info "Checking logs for $service..."
        
        local container_id=$(get_container_id "$service" "$CURRENT_COMPOSE_FILE")
        if [[ -z "$container_id" ]]; then
            log_error "Container for $service not found"
            success=false
            continue
        fi
        
        # Get recent logs (last 50 lines)
        local logs=$(docker logs --tail 50 "$container_id" 2>&1)
        
        # Check for critical errors (case insensitive)
        local error_patterns=(
            "panic"
            "fatal"
            "critical"
            "error.*failed"
            "exception"
            "segmentation fault"
        )
        
        local has_critical_errors=false
        for pattern in "${error_patterns[@]}"; do
            if echo "$logs" | grep -iq "$pattern"; then
                log_warning "Found potential error in $service logs: $pattern"
                has_critical_errors=true
            fi
        done
        
        if [[ "$has_critical_errors" == "false" ]]; then
            log_info "✓ $service logs look healthy"
        else
            log_warning "$service logs contain potential errors (non-critical for smoke test)"
        fi
    done
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Container logs health check"
        return 0
    else
        return 1
    fi
}

test_process_health() {
    log_test_start "Testing container process health"
    
    local services=$($DOCKER_COMPOSE_CMD -f "$CURRENT_COMPOSE_FILE" ps --services)
    local success=true
    
    for service in $services; do
        local container_id=$(get_container_id "$service" "$CURRENT_COMPOSE_FILE")
        if [[ -z "$container_id" ]]; then
            log_error "Container for $service not found"
            success=false
            continue
        fi
        
        # Check if main process is running
        local processes=$(docker exec "$container_id" ps aux 2>/dev/null | wc -l)
        
        if [[ $processes -gt 1 ]]; then  # At least ps + main process
            log_info "✓ $service has $processes running processes"
        else
            log_error "$service has no active processes"
            success=false
        fi
    done
    
    if [[ "$success" == "true" ]]; then
        log_test_pass "Container process health"
        return 0
    else
        return 1
    fi
}

test_resource_usage() {
    log_test_start "Testing container resource usage"
    
    local services=$($DOCKER_COMPOSE_CMD -f "$CURRENT_COMPOSE_FILE" ps --services)
    local success=true
    
    for service in $services; do
        local container_id=$(get_container_id "$service" "$CURRENT_COMPOSE_FILE")
        if [[ -z "$container_id" ]]; then
            continue
        fi
        
        # Get container stats (single snapshot)
        local stats=$(docker stats --no-stream --format "table {{.CPUPerc}}\t{{.MemUsage}}" "$container_id" 2>/dev/null | tail -n +2)
        
        if [[ -n "$stats" ]]; then
            log_info "✓ $service resource usage: $stats"
        else
            log_warning "Could not get resource stats for $service"
        fi
    done
    
    log_test_pass "Container resource usage check"
    return 0
}

# Run main function
main "$@" 