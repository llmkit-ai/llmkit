#!/bin/bash

# Test Helper Functions for Docker Testing

# Docker Compose version detection (same logic as build.sh)
DOCKER_COMPOSE_CMD="docker compose"
if ! docker compose version >/dev/null 2>&1; then
    DOCKER_COMPOSE_CMD="docker-compose"
fi

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

log_test_start() {
    echo -e "${BLUE}[TEST]${NC} Starting: $1"
    TESTS_RUN=$((TESTS_RUN + 1))
}

log_test_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_test_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

# Test summary
print_test_summary() {
    echo ""
    echo "========================================="
    echo "Test Summary:"
    echo "  Total:  $TESTS_RUN"
    echo "  Passed: $TESTS_PASSED"
    echo "  Failed: $TESTS_FAILED"
    echo "========================================="
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

# Timing functions
start_timer() {
    TIMER_START=$(date +%s)
}

end_timer() {
    TIMER_END=$(date +%s)
    TIMER_DIFF=$((TIMER_END - TIMER_START))
    echo "Time elapsed: ${TIMER_DIFF}s"
}

# Timeout wrapper
with_timeout() {
    local timeout=$1
    shift
    timeout $timeout "$@"
    return $?
}

# Wait for service to be ready
wait_for_service() {
    local host=$1
    local port=$2
    local timeout=${3:-30}
    local interval=${4:-2}
    
    log_info "Waiting for service at $host:$port (timeout: ${timeout}s)"
    
    local elapsed=0
    while [[ $elapsed -lt $timeout ]]; do
        if curl -s -f "http://$host:$port" >/dev/null 2>&1; then
            log_success "Service $host:$port is ready"
            return 0
        fi
        sleep $interval
        elapsed=$((elapsed + interval))
    done
    
    log_error "Service $host:$port not ready after ${timeout}s"
    return 1
}

# Wait for health endpoint
wait_for_health() {
    local url=$1
    local timeout=${2:-30}
    local interval=${3:-2}
    
    log_info "Waiting for health check at $url (timeout: ${timeout}s)"
    
    local elapsed=0
    while [[ $elapsed -lt $timeout ]]; do
        local response=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null)
        if [[ "$response" == "200" ]]; then
            log_success "Health check passed for $url"
            return 0
        fi
        sleep $interval
        elapsed=$((elapsed + interval))
    done
    
    log_error "Health check failed for $url after ${timeout}s"
    return 1
}

# Assert functions
assert_equals() {
    local expected=$1
    local actual=$2
    local message=${3:-"Assertion failed"}
    
    if [[ "$expected" == "$actual" ]]; then
        log_test_pass "$message: '$actual'"
        return 0
    else
        log_test_fail "$message: expected '$expected', got '$actual'"
        return 1
    fi
}

assert_contains() {
    local string=$1
    local substring=$2
    local message=${3:-"String contains assertion failed"}
    
    if [[ "$string" == *"$substring"* ]]; then
        log_test_pass "$message: found '$substring'"
        return 0
    else
        log_test_fail "$message: '$substring' not found in '$string'"
        return 1
    fi
}

assert_file_exists() {
    local file=$1
    local message=${2:-"File exists assertion failed"}
    
    if [[ -f "$file" ]]; then
        log_test_pass "$message: $file exists"
        return 0
    else
        log_test_fail "$message: $file does not exist"
        return 1
    fi
}

assert_command_success() {
    local command="$1"
    local message=${2:-"Command success assertion failed"}
    
    if eval "$command" >/dev/null 2>&1; then
        log_test_pass "$message: '$command' succeeded"
        return 0
    else
        log_test_fail "$message: '$command' failed"
        return 1
    fi
}

# Docker-specific helpers
get_container_id() {
    local service_name=$1
    local compose_file=${2:-"docker/docker-compose.dev.yml"}
    
    $DOCKER_COMPOSE_CMD -f "$compose_file" ps -q "$service_name"
}

get_container_status() {
    local container_id=$1
    docker inspect --format='{{.State.Status}}' "$container_id" 2>/dev/null
}

get_container_health() {
    local container_id=$1
    docker inspect --format='{{.State.Health.Status}}' "$container_id" 2>/dev/null
}

cleanup_containers() {
    local compose_file=${1:-"docker/docker-compose.dev.yml"}
    
    log_info "Cleaning up containers..."
    $DOCKER_COMPOSE_CMD -f "$compose_file" down --volumes --remove-orphans >/dev/null 2>&1 || true
}

cleanup_images() {
    local project_prefix=${1:-"llmkit"}
    
    log_info "Cleaning up project images..."
    docker images --format "table {{.Repository}}:{{.Tag}}" | grep "$project_prefix" | xargs -r docker rmi -f >/dev/null 2>&1 || true
}

cleanup_all() {
    log_info "Performing full cleanup..."
    cleanup_containers "docker/docker-compose.dev.yml"
    cleanup_containers "docker/docker-compose.yml"
    cleanup_images
    docker system prune -f >/dev/null 2>&1 || true
} 