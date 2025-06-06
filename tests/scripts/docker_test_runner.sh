#!/bin/bash

# Docker Test Runner
# Main script to orchestrate all Docker-related tests

set -e

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DOCKER_TESTS_DIR="$PROJECT_ROOT/tests/docker"

# Source test helpers
source "$DOCKER_TESTS_DIR/utils/test_helpers.sh"

# Test configuration
DEFAULT_TIMEOUT=1800  # 30 minutes total timeout
INDIVIDUAL_TEST_TIMEOUT=600  # 10 minutes per test suite

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

show_help() {
    cat << EOF
${BOLD}Docker Test Runner${NC}

${BOLD}USAGE:${NC}
    $0 [OPTIONS] [TEST_SUITE]

${BOLD}OPTIONS:${NC}
    --smoke              Run only smoke tests (quick validation)
    --unit               Run only unit tests (detailed functionality)
    --all                Run all tests (default)
    --parallel           Run test suites in parallel where possible
    --cleanup            Cleanup Docker resources before/after tests
    --no-cleanup         Skip cleanup (faster for development)
    --timeout <seconds>  Set overall timeout (default: $DEFAULT_TIMEOUT)
    --help, -h           Show this help message

${BOLD}TEST SUITES:${NC}
    smoke                All smoke tests
    unit                 All unit tests
    build                Build-related tests
    runtime              Runtime validation tests
    health               Health check tests
    cache                Cache functionality tests
    hot-reload           Hot reload functionality tests
    build-script         Build script unit tests

${BOLD}EXAMPLES:${NC}
    $0                           # Run all tests
    $0 --smoke                   # Run only smoke tests
    $0 --unit cache              # Run only cache unit tests
    $0 --parallel --cleanup      # Run all tests in parallel with cleanup
    $0 build                     # Run only build smoke tests

${BOLD}ENVIRONMENT:${NC}
    DOCKER_TEST_TIMEOUT          Override default timeout
    DOCKER_TEST_NO_CLEANUP       Skip cleanup if set to 'true'
    DOCKER_TEST_PARALLEL         Run in parallel if set to 'true'

EOF
}

main() {
    # Parse command line arguments
    local test_mode="all"
    local specific_suite=""
    local cleanup_mode="auto"
    local parallel_mode=false
    local timeout=$DEFAULT_TIMEOUT
    
    # Check environment variables
    if [[ -n "$DOCKER_TEST_TIMEOUT" ]]; then
        timeout=$DOCKER_TEST_TIMEOUT
    fi
    
    if [[ "$DOCKER_TEST_NO_CLEANUP" == "true" ]]; then
        cleanup_mode="none"
    fi
    
    if [[ "$DOCKER_TEST_PARALLEL" == "true" ]]; then
        parallel_mode=true
    fi
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --smoke)
                test_mode="smoke"
                shift
                ;;
            --unit)
                test_mode="unit"
                shift
                ;;
            --all)
                test_mode="all"
                shift
                ;;
            --parallel)
                parallel_mode=true
                shift
                ;;
            --cleanup)
                cleanup_mode="always"
                shift
                ;;
            --no-cleanup)
                cleanup_mode="none"
                shift
                ;;
            --timeout)
                timeout=$2
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            build|runtime|health|cache|hot-reload|build-script|smoke|unit)
                specific_suite=$1
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate timeout
    if ! [[ "$timeout" =~ ^[0-9]+$ ]]; then
        log_error "Invalid timeout value: $timeout"
        exit 1
    fi
    
    # Print test configuration
    print_test_header "$test_mode" "$specific_suite" "$cleanup_mode" "$parallel_mode" "$timeout"
    
    # Ensure we're in the project root
    cd "$PROJECT_ROOT"
    
    # Pre-test cleanup
    if [[ "$cleanup_mode" == "always" ]]; then
        perform_cleanup "pre-test"
    fi
    
    # Run tests with timeout
    if ! run_tests "$test_mode" "$specific_suite" "$parallel_mode"; then
        echo ""
        echo -e "${BOLD}${RED}================================${NC}"
        echo -e "${BOLD}${RED}❌ ERROR - Docker Tests FAILED! ❌${NC}"
        echo -e "${BOLD}${RED}================================${NC}"
        echo ""
        log_error "Tests failed - check output above for details"
        
        # Post-failure cleanup
        if [[ "$cleanup_mode" != "none" ]]; then
            perform_cleanup "post-failure"
        fi
        
        exit 1
    fi
    
    # Post-test cleanup
    if [[ "$cleanup_mode" == "always" ]]; then
        perform_cleanup "post-test"
    fi
    
    # Print final summary
    print_final_summary
}

print_test_header() {
    local mode=$1
    local suite=$2
    local cleanup=$3
    local parallel=$4
    local timeout=$5
    
    echo -e "${BOLD}${BLUE}================================${NC}"
    echo -e "${BOLD}${BLUE}     Docker Test Runner${NC}"
    echo -e "${BOLD}${BLUE}================================${NC}"
    echo ""
    echo -e "${BOLD}Configuration:${NC}"
    echo "  Mode:      $mode"
    echo "  Suite:     ${suite:-all}"
    echo "  Cleanup:   $cleanup"
    echo "  Parallel:  $parallel"
    echo "  Timeout:   ${timeout}s"
    echo "  Project:   $PROJECT_ROOT"
    echo ""
}

run_tests() {
    local mode=$1
    local suite=$2
    local parallel=$3
    
    case $mode in
        "smoke")
            run_smoke_tests "$suite" "$parallel"
            ;;
        "unit")
            run_unit_tests "$suite" "$parallel"
            ;;
        "all")
            run_all_tests "$suite" "$parallel"
            ;;
        *)
            log_error "Invalid test mode: $mode"
            return 1
            ;;
    esac
}

run_smoke_tests() {
    local suite=$1
    local parallel=$2
    
    log_info "Running Docker Smoke Tests"
    
    local smoke_tests=()
    
    case $suite in
        "build"|"")
            smoke_tests+=("$DOCKER_TESTS_DIR/smoke/build_test.sh")
            ;;
        "runtime"|"")
            smoke_tests+=("$DOCKER_TESTS_DIR/smoke/runtime_test.sh")
            ;;
        "health"|"")
            smoke_tests+=("$DOCKER_TESTS_DIR/smoke/health_check.sh")
            ;;
        "smoke"|"")
            smoke_tests=(
                "$DOCKER_TESTS_DIR/smoke/build_test.sh"
                "$DOCKER_TESTS_DIR/smoke/runtime_test.sh"
                "$DOCKER_TESTS_DIR/smoke/health_check.sh"
            )
            ;;
        *)
            log_error "Invalid smoke test suite: $suite"
            return 1
            ;;
    esac
    
    if [[ "$parallel" == "true" && ${#smoke_tests[@]} -gt 1 ]]; then
        run_tests_parallel "${smoke_tests[@]}"
    else
        run_tests_sequential "${smoke_tests[@]}"
    fi
}

run_unit_tests() {
    local suite=$1
    local parallel=$2
    
    log_info "Running Docker Unit Tests"
    
    local unit_tests=()
    
    case $suite in
        "build-script"|"")
            unit_tests+=("$DOCKER_TESTS_DIR/unit/build_script_test.sh")
            ;;
        "cache"|"")
            unit_tests+=("$DOCKER_TESTS_DIR/unit/cache_test.sh")
            ;;
        "hot-reload"|"")
            unit_tests+=("$DOCKER_TESTS_DIR/unit/hot_reload_test.sh")
            ;;
        "unit"|"")
            unit_tests=(
                "$DOCKER_TESTS_DIR/unit/build_script_test.sh"
                "$DOCKER_TESTS_DIR/unit/cache_test.sh"
                "$DOCKER_TESTS_DIR/unit/hot_reload_test.sh"
            )
            ;;
        *)
            log_error "Invalid unit test suite: $suite"
            return 1
            ;;
    esac
    
    if [[ "$parallel" == "true" && ${#unit_tests[@]} -gt 1 ]]; then
        run_tests_parallel "${unit_tests[@]}"
    else
        run_tests_sequential "${unit_tests[@]}"
    fi
}

run_all_tests() {
    local suite=$1
    local parallel=$2
    
    log_info "Running All Docker Tests"
    
    if [[ -n "$suite" ]]; then
        # Run specific suite from both smoke and unit tests
        run_smoke_tests "$suite" false
        run_unit_tests "$suite" false
    else
        # Run all tests
        if [[ "$parallel" == "true" ]]; then
            # Run smoke tests first, then unit tests
            run_smoke_tests "" false
            run_unit_tests "" true
        else
            run_smoke_tests "" false
            run_unit_tests "" false
        fi
    fi
}

run_tests_sequential() {
    local tests=("$@")
    
    # Export individual test timeout for use by test scripts
    export INDIVIDUAL_TEST_TIMEOUT
    
    for test_script in "${tests[@]}"; do
        if [[ -f "$test_script" && -x "$test_script" ]]; then
            local test_name=$(basename "$test_script" .sh)
            log_info "Running test: $test_name"
            
            start_timer
            # Pass cleanup_mode as first parameter to test script
            if with_timeout "$INDIVIDUAL_TEST_TIMEOUT" "$test_script" "$cleanup_mode"; then
                end_timer
                log_success "Test $test_name completed successfully (${TIMER_DIFF}s)"
            else
                end_timer
                log_error "Test $test_name failed or timed out (${TIMER_DIFF}s)"
                return 1
            fi
        else
            log_error "Test script not found or not executable: $test_script"
            return 1
        fi
    done
}

run_tests_parallel() {
    local tests=("$@")
    local pids=()
    local test_results=()
    
    log_info "Running ${#tests[@]} tests in parallel"
    
    # Start all tests in background
    for test_script in "${tests[@]}"; do
        if [[ -f "$test_script" && -x "$test_script" ]]; then
            local test_name=$(basename "$test_script" .sh)
            log_info "Starting parallel test: $test_name"
            
            # Run test in background and capture PID
            (
                if with_timeout "$INDIVIDUAL_TEST_TIMEOUT" "$test_script" "$cleanup_mode" >/dev/null 2>&1; then
                    echo "PASS:$test_name"
                else
                    echo "FAIL:$test_name"
                fi
            ) &
            pids+=($!)
        else
            log_error "Test script not found or not executable: $test_script"
            return 1
        fi
    done
    
    # Wait for all tests to complete
    local all_passed=true
    for i in "${!pids[@]}"; do
        local pid=${pids[$i]}
        if wait $pid; then
            local test_name=$(basename "${tests[$i]}" .sh)
            log_success "Parallel test $test_name completed successfully"
        else
            local test_name=$(basename "${tests[$i]}" .sh)
            log_error "Parallel test $test_name failed"
            all_passed=false
        fi
    done
    
    if [[ "$all_passed" == "true" ]]; then
        log_success "All parallel tests completed successfully"
        return 0
    else
        log_error "Some parallel tests failed"
        return 1
    fi
}

perform_cleanup() {
    local stage=$1
    
    log_info "Performing Docker cleanup ($stage)"
    
    # Source cleanup functions
    source "$DOCKER_TESTS_DIR/utils/docker_helpers.sh"
    
    case $stage in
        "pre-test")
            log_info "Pre-test cleanup: Removing existing containers and images"
            cleanup_all
            ;;
        "post-test")
            log_info "Post-test cleanup: Removing test containers and images"
            cleanup_all
            ;;
        "post-failure")
            log_info "Post-failure cleanup: Emergency cleanup"
            cleanup_all
            ;;
    esac
    
    log_success "Cleanup completed"
}

print_final_summary() {
    echo ""
    echo -e "${BOLD}${GREEN}================================${NC}"
    echo -e "${BOLD}${GREEN}     Test Execution Complete${NC}"
    echo -e "${BOLD}${GREEN}================================${NC}"
    echo ""
    echo -e "${BOLD}${GREEN}🎉 ALL GOOD - Docker Pipeline Tests PASSED! 🎉${NC}"
    echo ""
    echo "Test execution completed successfully!"
    echo "Check individual test outputs above for detailed results."
    echo ""
    echo "To run specific tests:"
    echo "  $0 --smoke              # Quick validation"
    echo "  $0 --unit cache         # Cache functionality"
    echo "  $0 --unit hot-reload    # Hot reload testing"
    echo ""
}

# Ensure all test scripts are executable
make_scripts_executable() {
    log_info "Ensuring test scripts are executable"
    
    find "$DOCKER_TESTS_DIR" -name "*.sh" -type f -exec chmod +x {} \;
    
    log_success "Test scripts are executable"
}

# Check prerequisites
check_prerequisites() {
    local missing_deps=()
    
    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        missing_deps+=("docker")
    fi
    
    # Check Docker Compose
    if ! docker compose version >/dev/null 2>&1; then
        if ! command -v docker-compose >/dev/null 2>&1; then
            missing_deps+=("docker-compose")
        fi
    fi
    
    # Check curl for connectivity tests
    if ! command -v curl >/dev/null 2>&1; then
        missing_deps+=("curl")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi
    
    log_success "All prerequisites are available"
}

# Initialize
initialize() {
    # Check prerequisites
    check_prerequisites
    
    # Make scripts executable
    make_scripts_executable
    
    log_success "Initialization completed"
}

# Cleanup on exit
cleanup_on_exit() {
    local exit_code=$?
    
    if [[ $exit_code -ne 0 ]]; then
        log_warning "Test runner exiting with error code $exit_code"
        
        # Emergency cleanup if needed
        if [[ "$cleanup_mode" != "none" ]]; then
            perform_cleanup "emergency" >/dev/null 2>&1 || true
        fi
    fi
    
    exit $exit_code
}

# Set up trap for cleanup
trap cleanup_on_exit EXIT

# Initialize and run
initialize
main "$@" 