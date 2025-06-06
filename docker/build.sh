#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' 

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Change to project root
cd "$PROJECT_ROOT"

# Load environment variables from .env file if it exists
if [[ -f ".env" ]]; then
    print_status "Loading environment variables from .env file"
    set -a  # automatically export all variables
    source .env
    set +a  # stop automatically exporting
else
    print_warning "No .env file found in project root"
fi

# Install and enable Docker BuildKit for  caching user might need to be root so might want to ask via cli but its kinda the whole point tbd
if ! docker buildx version >/dev/null 2>&1; then
    print_status "Installing Docker buildx for optimal caching..."
    mkdir -p ~/.docker/cli-plugins/
    curl -SL https://github.com/docker/buildx/releases/download/v0.12.1/buildx-v0.12.1.linux-amd64 -o ~/.docker/cli-plugins/docker-buildx
    chmod a+x ~/.docker/cli-plugins/docker-buildx
    print_status "Docker buildx installed successfully"
else
    print_status "Docker buildx already available"
fi

# Enable Docker BuildKit for cache mounts and optimizations
export DOCKER_BUILDKIT=1
print_status "Docker BuildKit enabled for optimal caching"

# Default values (to be discussed with dan)
MODE="dev"
FORCE_REBUILD=false
NO_CACHE=false

# Parse command line arguments
# could use some love, i would throw it in cluade code but dont have time to do it rn (good issue for noobies)
while [[ $# -gt 0 ]]; do
    case $1 in
        --prod|--production)
            MODE="prod"
            shift
            ;;
        --dev|--development)
            MODE="dev"
            shift
            ;;
        --rebuild)
            FORCE_REBUILD=true
            shift
            ;;
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--dev|--prod] [--rebuild] [--no-cache]"
            echo ""
            echo "Options:"
            echo "  --dev, --development    Build for development (default)"
            echo "  --prod, --production    Build for production"
            echo "  --rebuild               Force rebuild of all images"
            echo "  --no-cache              Build without using cache"
            echo "  --help, -h              Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_status "Building in $MODE mode from $(pwd)..."

# Build command options
BUILD_OPTS=""
if [[ "$NO_CACHE" == "true" ]]; then
    BUILD_OPTS="$BUILD_OPTS --no-cache"
fi

if [[ "$FORCE_REBUILD" == "true" ]]; then
    BUILD_OPTS="$BUILD_OPTS --force-recreate"
fi

if [[ "$MODE" == "dev" ]]; then
    COMPOSE_FILE="docker/docker-compose.dev.yml"
    print_status "Using development configuration with hot reloading"
else
    COMPOSE_FILE="docker/docker-compose.yml"
    print_status "Using production configuration"
fi

if [[ ! -f "$COMPOSE_FILE" ]]; then
    print_error "Docker compose file not found: $COMPOSE_FILE"
    exit 1
fi

BUN_LOCK_HASH=$(sha256sum ui/bun.lock 2>/dev/null | cut -d' ' -f1 || echo "no-lock")
export BUN_LOCK_HASH

DOCKER_COMPOSE_CMD="docker compose"
if ! docker compose version >/dev/null 2>&1; then
    DOCKER_COMPOSE_CMD="docker-compose"
    print_warning "Using legacy docker-compose v1 (consider upgrading to Docker Compose v2)"
fi

# Build and start services
print_status "Building services..."
$DOCKER_COMPOSE_CMD -f "$COMPOSE_FILE" build $BUILD_OPTS

print_status "Starting services..."
$DOCKER_COMPOSE_CMD -f "$COMPOSE_FILE" up -d

print_status "Services started successfully!"
print_status "Backend: http://localhost:${BACKEND_PORT:-8000}"
print_status "UI: http://localhost:${UI_PORT:-3000}"

if [[ "$MODE" == "dev" ]]; then
    print_warning "Development mode: Changes to source files will trigger automatic rebuilds"
    print_status "To view logs: $DOCKER_COMPOSE_CMD -f $COMPOSE_FILE logs -f"
    print_status "To stop: $DOCKER_COMPOSE_CMD -f $COMPOSE_FILE down"
fi 