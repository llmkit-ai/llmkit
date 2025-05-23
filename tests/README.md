# Docker Testing Suite

A comprehensive testing framework for validating Docker build optimization, hot reload functionality, and deployment workflows.

## Overview

This testing suite validates the Docker infrastructure described in `docker/docker_optimization.md`, including:

- **Build Performance**: cargo-chef, sccache, and BuildKit optimizations
- **Hot Reload**: Development workflow with cargo-watch and Bun dev server
- **Cache Effectiveness**: Multi-layer caching and volume persistence
- **Runtime Health**: Service connectivity and container health checks
- **Environment Configuration**: Proper volume mounts and environment variables

## Quick Start

```bash
# Run all tests (recommended first run)
./tests/scripts/docker_test_runner.sh

# Run only smoke tests (fast validation)
./tests/scripts/docker_test_runner.sh --smoke

# Run specific test suite
./tests/scripts/docker_test_runner.sh --unit cache

# Run with cleanup (force cleanup before/after)
./tests/scripts/docker_test_runner.sh --cleanup

# Run without cleanup (preserve containers on failure/timeout)
./tests/scripts/docker_test_runner.sh --no-cleanup

# Longer timeout for slow builds (like first-time cargo install)
./tests/scripts/docker_test_runner.sh --smoke --no-cleanup --timeout 1200
```

## Test Structure

```
tests/
├── docker/                     # Docker-specific tests
│   ├── smoke/                  # Quick validation tests
│   │   ├── build_test.sh       # Build functionality
│   │   ├── runtime_test.sh     # Runtime validation
│   │   └── health_check.sh     # Service health checks
│   ├── unit/                   # Detailed functionality tests
│   │   ├── build_script_test.sh    # Build script validation
│   │   ├── cache_test.sh           # Cache effectiveness
│   │   └── hot_reload_test.sh      # Hot reload functionality
│   └── utils/                  # Shared utilities
│       ├── test_helpers.sh     # General test helpers
│       └── docker_helpers.sh   # Docker-specific helpers
├── scripts/
│   └── docker_test_runner.sh   # Main test orchestrator
└── README.md                   # This file
```

## Test Types

### Smoke Tests (Quick Validation)

**Purpose**: Fast validation that basic functionality works
**Runtime**: ~5-10 minutes
**Use Case**: Pre-commit checks, CI validation

#### `build_test.sh`

- ✅ Build script validation
- ✅ Docker Compose file syntax
- ✅ Development build functionality
- ✅ Production build functionality
- ✅ Container startup verification

#### `runtime_test.sh`

- ✅ Service port accessibility
- ✅ Container health status
- ✅ Basic HTTP connectivity
- ✅ Volume mount validation
- ✅ Environment variable verification

#### `health_check.sh`

- ✅ Service health endpoints
- ✅ Response time validation
- ✅ Container log analysis
- ✅ Process health checks
- ✅ Resource usage monitoring

### Unit Tests (Detailed Validation)

**Purpose**: Comprehensive validation of specific components
**Runtime**: ~15-30 minutes
**Use Case**: Feature development, optimization validation

#### `build_script_test.sh`

- ✅ Script existence and permissions
- ✅ Help functionality
- ✅ Argument parsing logic
- ✅ Invalid argument handling
- ✅ Environment variable loading
- ✅ Docker BuildKit detection
- ✅ Compose file selection logic
- ✅ Error handling and cleanup

#### `cache_test.sh`

- ✅ Dockerfile cache optimization structure
- ✅ Cargo-chef implementation
- ✅ Bun cache implementation
- ✅ Build cache effectiveness (timing)
- ✅ Docker layer caching analysis
- ✅ Volume cache persistence
- ✅ BuildKit cache mounts
- ✅ Cache invalidation behavior

#### `hot_reload_test.sh`

- ✅ Cargo-watch setup verification
- ✅ Bun dev server configuration
- ✅ Backend hot reload functionality
- ✅ UI hot reload functionality
- ✅ Volume mount configuration
- ✅ Development command overrides
- ✅ Reload performance measurement
- ✅ Error recovery during reload

## Usage Examples

### Development Workflow

```bash
# Quick validation during development
./tests/scripts/docker_test_runner.sh --smoke

# Test specific functionality you're working on
./tests/scripts/docker_test_runner.sh --unit hot-reload

# Full validation before pushing
./tests/scripts/docker_test_runner.sh --all --cleanup

# Preserve containers when testing/debugging failures
./tests/scripts/docker_test_runner.sh --smoke --no-cleanup

# First-time run or slow build systems (extended timeout)
./tests/scripts/docker_test_runner.sh --smoke --no-cleanup --timeout 1200
```

### CI/CD Integration

```bash
# Fast CI pipeline
./tests/scripts/docker_test_runner.sh --smoke --parallel --timeout 600

# Comprehensive validation
./tests/scripts/docker_test_runner.sh --all --cleanup --timeout 1800
```

### Performance Validation

```bash
# Test cache effectiveness
./tests/scripts/docker_test_runner.sh --unit cache

# Validate hot reload performance
./tests/scripts/docker_test_runner.sh --unit hot-reload
```

## Command Reference

### Main Test Runner

```bash
./tests/scripts/docker_test_runner.sh [OPTIONS] [TEST_SUITE]
```

#### Options

- `--smoke`: Run only smoke tests (quick validation)
- `--unit`: Run only unit tests (detailed functionality)
- `--all`: Run all tests (default)
- `--parallel`: Run test suites in parallel where possible
- `--cleanup`: Cleanup Docker resources before/after tests
- `--no-cleanup`: Skip cleanup (faster for development, preserves containers on failure)
- `--timeout <seconds>`: Set overall timeout (default: 1800s)
- `--help`, `-h`: Show help message

#### Test Suites

- `smoke`: All smoke tests
- `unit`: All unit tests
- `build`: Build-related tests
- `runtime`: Runtime validation tests
- `health`: Health check tests
- `cache`: Cache functionality tests
- `hot-reload`: Hot reload functionality tests
- `build-script`: Build script unit tests

### Individual Test Scripts

```bash
# Run individual smoke tests
./tests/docker/smoke/build_test.sh
./tests/docker/smoke/runtime_test.sh
./tests/docker/smoke/health_check.sh

# Run individual unit tests
./tests/docker/unit/build_script_test.sh
./tests/docker/unit/cache_test.sh
./tests/docker/unit/hot_reload_test.sh
```

## Environment Variables

Configure test behavior with environment variables:

```bash
# Override default timeout (30 minutes)
export DOCKER_TEST_TIMEOUT=1200

# Skip cleanup for faster development
export DOCKER_TEST_NO_CLEANUP=true

# Enable parallel execution
export DOCKER_TEST_PARALLEL=true

# Run tests
./tests/scripts/docker_test_runner.sh
```

## Test Configuration

### Performance Thresholds

The tests validate against these performance benchmarks:

- **Cached Build Time**: < 60 seconds
- **Hot Reload Time**: < 15 seconds
- **Service Response Time**: < 5 seconds
- **Health Check Timeout**: 45 seconds

### Cache Validation

Tests verify:

- Cold build vs warm build performance improvement
- Layer caching effectiveness
- Volume persistence across container restarts
- BuildKit cache mount utilization

### Hot Reload Validation

Tests verify:

- File change detection within 45 seconds
- Successful compilation after changes
- Error recovery from syntax errors
- Volume mount configuration for live updates

## Cleanup Modes

The test runner supports different cleanup behaviors to optimize development workflow:

### **Auto Mode (Default)**

```bash
./tests/scripts/docker_test_runner.sh --smoke
```

- Cleans up containers/images on test failure
- Preserves Docker layer cache for faster rebuilds
- Good for most CI/development scenarios

### **No Cleanup Mode**

```bash
./tests/scripts/docker_test_runner.sh --smoke --no-cleanup
```

- **Never destroys containers or images**
- Preserves everything even on timeout/failure
- Perfect for debugging build issues
- **Recommended for first-time runs** (cargo install takes 5+ minutes)

### **Force Cleanup Mode**

```bash
./tests/scripts/docker_test_runner.sh --smoke --cleanup
```

- Always cleans up before and after tests
- Ensures completely clean state
- Good for production CI validation

## Troubleshooting

### Common Issues

#### First-Time Build Timeouts

The first run often times out because `cargo install sqlx-cli` and `cargo install cargo-watch` are slow:

```bash
# Solution: Use extended timeout + no cleanup
./tests/scripts/docker_test_runner.sh --smoke --no-cleanup --timeout 1200

# This preserves the build progress even if it times out
# Subsequent runs will be much faster
```

#### Build Failures

```bash
# Check Docker daemon
docker info

# Verify Docker Compose
docker compose version

# Clean Docker state
docker system prune -a --volumes
```

#### Permission Issues

```bash
# Fix script permissions
chmod +x tests/scripts/docker_test_runner.sh
chmod +x tests/docker/**/*.sh

# Fix file ownership
sudo chown -R $USER:$USER .
```

#### Test Timeouts

```bash
# Increase timeout for slow systems
export DOCKER_TEST_TIMEOUT=3600

# Run without cleanup for faster iterations
./tests/scripts/docker_test_runner.sh --no-cleanup
```

#### Hot Reload Issues

```bash
# Verify volume mounts are read-write
docker compose -f docker/docker-compose.dev.yml config

# Check container logs
docker compose -f docker/docker-compose.dev.yml logs -f
```

### Debug Mode

Enable verbose output for debugging:

```bash
# Enable debug output in individual tests
export DEBUG=true
./tests/docker/unit/cache_test.sh

# Run with detailed Docker output
export DOCKER_BUILDKIT_PROGRESS=plain
./tests/scripts/docker_test_runner.sh
```

## Integration with CI

### GitHub Actions Example

```yaml
- name: Run Docker Tests
  run: |
    ./tests/scripts/docker_test_runner.sh --smoke --parallel --timeout 900

- name: Run Full Docker Validation
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
  run: |
    ./tests/scripts/docker_test_runner.sh --all --cleanup --timeout 1800
```

### GitLab CI Example

```yaml
docker-smoke-tests:
  script:
    - ./tests/scripts/docker_test_runner.sh --smoke --parallel
  timeout: 15m

docker-full-tests:
  script:
    - ./tests/scripts/docker_test_runner.sh --all --cleanup
  timeout: 30m
  only:
    - main
```

## Contributing

### Adding New Tests

1. **Smoke Tests**: Add to `tests/docker/smoke/` for quick validation
2. **Unit Tests**: Add to `tests/docker/unit/` for detailed functionality
3. **Helpers**: Add shared functions to `tests/docker/utils/`

### Test Naming Convention

- `*_test.sh`: Test files
- `test_*()`: Test functions
- `*_helpers.sh`: Helper utilities

### Test Structure Template

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../utils/docker_helpers.sh"

main() {
    log_info "Starting [Test Name]"

    # Test setup
    cd "$PROJECT_ROOT"

    # Run tests
    test_functionality_1
    test_functionality_2

    print_test_summary
}

test_functionality_1() {
    log_test_start "Testing functionality 1"

    # Test implementation
    if [[ condition ]]; then
        log_test_pass "Functionality 1 working"
        return 0
    else
        log_test_fail "Functionality 1 failed"
        return 1
    fi
}

# Cleanup on exit
trap 'cleanup_all' EXIT

# Run main function
main "$@"
```

## Performance Benchmarks

Based on the optimization goals in `docker/docker_optimization.md`:

### Build Time Improvements

- **Before Optimization**: 8-12 minutes
- **After Optimization**: 2-4 minutes
- **Target Improvement**: ~70% reduction

### Incremental Build Times

- **Code Changes**: < 15 seconds
- **Dependency Changes**: 1-3 minutes
- **Clean Rebuild**: 2-4 minutes

### Cache Effectiveness

- **Layer Reuse**: > 80% of layers cached
- **Volume Persistence**: Cargo target cache persists
- **BuildKit Optimization**: sccache provides < 10s rebuilds

The testing suite validates these benchmarks and alerts when performance degrades.
