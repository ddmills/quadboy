#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

HELP_STRING=$(
	cat <<-END
		usage: build-web.sh [OPTIONS]

		Build QuadBoy for the web using WASM.

		OPTIONS:
			--release, -r	Build in release mode (optimized)
			--serve, -s	Start local server after build
			--open, -o	Open browser after build (requires --serve)
			--help, -h	Show this help message

		EXAMPLES:
			./build-web.sh		# Debug build
			./build-web.sh -r	# Release build
			./build-web.sh -rs	# Release build with server
			./build-web.sh -rso	# Release, serve, and open browser

		Version: 1.0
	END
)

# Progress indicator
progress() {
	echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

# Success message
success() {
	echo -e "${GREEN}✓${NC} $1"
}

# Warning message
warn() {
	echo -e "${YELLOW}⚠${NC} $1"
}

# Error message and exit
die() {
	echo -e "${RED}✗ Error:${NC} $*" >&2
	echo >&2
	echo "$HELP_STRING" >&2
	exit 1
}

# Check if command exists
check_command() {
	if ! command -v "$1" &> /dev/null; then
		die "$1 is required but not installed"
	fi
}

# Initialize variables
RELEASE=""
SERVE=""
OPEN=""
PROJECT_NAME="quadboy"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
	case $1 in
		-r|--release)
			RELEASE="yes"
			shift
			;;
		-s|--serve)
			SERVE="yes"
			shift
			;;
		-o|--open)
			OPEN="yes"
			shift
			;;
		-h|--help)
			echo "$HELP_STRING"
			exit 0
			;;
		-*)
			# Handle combined short options like -rso
			if [[ ${1:1:1} != "-" ]]; then
				for (( i=1; i<${#1}; i++ )); do
					case ${1:$i:1} in
						r) RELEASE="yes" ;;
						s) SERVE="yes" ;;
						o) OPEN="yes" ;;
						*) die "Unknown option: -${1:$i:1}" ;;
					esac
				done
			else
				die "Unknown option: $1"
			fi
			shift
			;;
		*)
			POSITIONAL+=("$1")
			shift
			;;
	esac
done

# Validate option combinations
if [[ -n "$OPEN" && -z "$SERVE" ]]; then
	die "--open requires --serve"
fi

# Validate environment and dependencies
progress "Validating build environment..."

# Check required commands
check_command "cargo"
check_command "wasm-bindgen"

# Check if wasm32 target is installed
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
	warn "wasm32-unknown-unknown target not installed. Installing..."
	rustup target add wasm32-unknown-unknown || die "Failed to install wasm32-unknown-unknown target"
	success "wasm32-unknown-unknown target installed"
fi

# Get build information
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_TIME=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
BUILD_MODE="${RELEASE:+release}"
BUILD_MODE="${BUILD_MODE:-debug}"

progress "Building ${PROJECT_NAME} (${BUILD_MODE} mode, git: ${GIT_SHA})"

# Always clean dist directory
progress "Cleaning dist directory..."
rm -rf dist
success "Cleaned dist directory"

HTML=$(
	cat <<-END
		<html lang="en">
		<head>
			<meta charset="utf-8">
			<title>${PROJECT_NAME}</title>
			<style>
				html,
				body,
				canvas {
					margin: 0px;
					padding: 0px;
					width: 100%;
					height: 100%;
					overflow: hidden;
					position: absolute;
					z-index: 0;
					image-rendering: pixelated;
					background-color: black;
				}
			</style>
		</head>
		<body style="margin: 0; padding: 0; height: 100vh; width: 100vw;">
			<canvas id="glcanvas" tabindex='1' hidden></canvas>
			<script src="./mq_js_bundle.js"></script>
			<script type="module">
				console.log('%c ${PROJECT_NAME} - [${GIT_SHA} ${BUILD_MODE}] ${BUILD_TIME} ', 'background: #111411; color: #e9e548;');

				import init, { set_wasm } from "./${PROJECT_NAME}.js";
				async function impl_run() {
					let wbg = await init();
					miniquad_add_plugin({
						register_plugin: (a) => (a.wbg = wbg),
						on_init: () => set_wasm(wasm_exports),
						version: "0.0.1",
						name: "wbg",
					});
					load("./${PROJECT_NAME}_bg.wasm");
				}
				window.run = function() {
					document.getElementById("run-container").remove();
					document.getElementById("glcanvas").removeAttribute("hidden");
					document.getElementById("glcanvas").focus();
					impl_run();
				}
			</script>
			<div id="run-container" style="display: flex; justify-content: center; align-items: center; height: 100%; flex-direction: column;">
				<button onclick="run()">Run Game</button>
			</div>
		</body>
		</html>
	END
)

# Build Rust project
TARGET_DIR="target/wasm32-unknown-unknown"
if [[ -n "$RELEASE" ]]; then
	progress "Building Rust project (release mode)..."
	cargo build --release --target wasm32-unknown-unknown || die "Cargo build failed"
	TARGET_DIR="$TARGET_DIR/release"
else
	progress "Building Rust project (debug mode)..."
	cargo build --target wasm32-unknown-unknown || die "Cargo build failed"
	TARGET_DIR="$TARGET_DIR/debug"
fi
success "Rust build completed"

# Verify WASM file exists
WASM_FILE="$TARGET_DIR/$PROJECT_NAME.wasm"
if [[ ! -f "$WASM_FILE" ]]; then
	die "WASM file not found: $WASM_FILE"
fi

# Create output directory
progress "Setting up output directory..."
mkdir -p dist
success "Output directory ready"

# Generate wasm-bindgen bindings
progress "Generating WASM bindings..."
wasm-bindgen "$WASM_FILE" \
	--out-dir dist \
	--target web \
	--no-typescript \
	--weak-refs || die "wasm-bindgen failed"
success "WASM bindings generated"

# Apply Macroquad-specific patches
progress "Applying Macroquad patches..."
sed -i "s/import \* as __wbg_star0 from 'env';//" "dist/$PROJECT_NAME.js"
sed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" "dist/$PROJECT_NAME.js"
sed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg;/" "dist/$PROJECT_NAME.js"
sed -i "s/const imports = __wbg_get_imports();/return __wbg_get_imports();/" "dist/$PROJECT_NAME.js"
success "Patches applied"

# Copy assets and dependencies
progress "Copying assets and dependencies..."
echo "$HTML" > dist/index.html

if [[ -d "src/assets" ]]; then
	mkdir -p dist/src
	cp -r ./src/assets ./dist/src/
	success "Assets copied"
else
	warn "No assets directory found (src/assets)"
fi

if [[ -f "mq_js_bundle.js" ]]; then
	cp ./mq_js_bundle.js ./dist/
	success "Dependencies copied"
else
	warn "mq_js_bundle.js not found"
fi

# Get file sizes for reporting
WASM_SIZE=$(wc -c < "dist/${PROJECT_NAME}_bg.wasm" | tr -d ' ')
JS_SIZE=$(wc -c < "dist/${PROJECT_NAME}.js" | tr -d ' ')

# Format file sizes
format_size() {
	local size=$1
	if [[ $size -gt 1048576 ]]; then
		echo "$(( size / 1048576 ))MB"
	elif [[ $size -gt 1024 ]]; then
		echo "$(( size / 1024 ))KB"
	else
		echo "${size}B"
	fi
}

echo
success "Build completed successfully!"
echo "  📦 WASM file: $(format_size $WASM_SIZE)"
echo "  📄 JS file:   $(format_size $JS_SIZE)"
echo "  📁 Output:	dist/"

# Start server if requested
if [[ -n "$SERVE" ]]; then
	progress "Starting local server..."

	# Check for basic-http-server
	if ! command -v basic-http-server &> /dev/null; then
		die "basic-http-server is required for --serve. Install with: cargo install basic-http-server"
	fi

	cd dist

	# Open browser if requested
	if [[ -n "$OPEN" ]]; then
		progress "Opening browser..."
		if command -v xdg-open &> /dev/null; then
			xdg-open "http://localhost:8000" &
		elif command -v open &> /dev/null; then
			open "http://localhost:8000" &
		elif command -v start &> /dev/null; then
			start "http://localhost:8000" &
		else
			warn "Could not auto-open browser. Navigate to http://localhost:8000"
		fi
	fi
	
	success "Server starting at http://localhost:8000"
	echo "Press Ctrl+C to stop the server"
	basic-http-server --addr 127.0.0.1:8000
else
	echo
	echo "To test your build:"
	echo "  cd dist && basic-http-server ."
	echo "Then open http://localhost:8000"
fi
