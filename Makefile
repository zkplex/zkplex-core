.PHONY: help wasm-build wasm-copy cli-build play-dev play-stop play-restart play-build clean clean-wasm clean-cli clean-play

# Default target
help:
	@echo "ZKPlex Docker Build Commands"
	@echo "=============================="
	@echo ""
	@echo "WASM Compilation:"
	@echo "  make wasm-build       - Build WASM in Docker and copy to ./pkg"
	@echo "  make wasm-copy        - Copy WASM artifacts from container to host"
	@echo ""
	@echo "CLI Build:"
	@echo "  make cli-build        - Build CLI in Docker (use ./zkplex script to run)"
	@echo ""
	@echo "Play Development:"
	@echo "  make play-dev         - Start Play dev server in Docker"
	@echo "  make play-stop        - Stop Play container"
	@echo "  make play-restart     - Restart Play (stop + start)"
	@echo "  make play-build       - Build Play for production (outputs to play/dist/)"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean            - Remove all Docker containers and images"
	@echo "  make clean-wasm       - Remove only WASM-related artifacts"
	@echo "  make clean-cli        - Remove only CLI-related artifacts"
	@echo "  make clean-play       - Remove Play build artifacts (dist/)"

# Build WASM in Docker and copy artifacts to host
# Always builds without cache to ensure fresh code
wasm-build:
	@echo "Building WASM in Docker (without cache for fresh build)..."
	@BUILD_ID="build-$$(date +%Y%m%d-%H%M%S)"; \
	echo "Build ID: $$BUILD_ID"; \
	docker build --no-cache \
		--build-arg BUILD_ID=$$BUILD_ID \
		-f Dockerfile.wasm -t zkplex-wasm-builder .
	@echo "Creating temporary container to copy artifacts..."
	docker create --name zkplex-wasm-temp zkplex-wasm-builder
	@echo "Copying WASM artifacts to ./pkg..."
	rm -rf pkg
	mkdir -p pkg
	docker cp zkplex-wasm-temp:/output/zkplex_core.js ./pkg/
	docker cp zkplex-wasm-temp:/output/zkplex_core_bg.wasm ./pkg/
	docker cp zkplex-wasm-temp:/output/zkplex_core.d.ts ./pkg/
	@echo "Creating build info file..."
	@BUILD_ID="build-$$(date +%Y%m%d-%H%M%S)"; \
	echo "{\"buildId\": \"$$BUILD_ID\", \"timestamp\": $$(date +%s)}" > pkg/build-info.json
	@echo "Cleaning up temporary container..."
	docker rm zkplex-wasm-temp
	@echo "✓ WASM build complete! Artifacts in ./pkg"
	@echo "  - zkplex_core.js (JS bindings)"
	@echo "  - zkplex_core_bg.wasm (WASM binary)"
	@echo "  - zkplex_core.d.ts (TypeScript definitions)"
	@echo ""
	@echo "💡 Tip: Restart Play to use new WASM: make play-restart"

# Copy WASM artifacts from existing container
wasm-copy:
	@echo "Copying WASM artifacts from container..."
	docker cp zkplex-wasm-temp:/output ./pkg
	@echo "✓ WASM artifacts copied to ./pkg"

# Build CLI in Docker
# Always builds without cache to ensure fresh code
cli-build:
	@echo "Building CLI in Docker (without cache for fresh build)..."
	@BUILD_ID="build-$$(date +%Y%m%d-%H%M%S)"; \
	echo "Build ID: $$BUILD_ID"; \
	docker build --no-cache \
		--build-arg BUILD_ID=$$BUILD_ID \
		-f Dockerfile.cli -t zkplex-cli .
	@echo "✓ CLI build complete!"
	@echo ""
	@echo "To run CLI: ./zkplex [options]"
	@echo "Examples:"
	@echo "  ./zkplex --version"
	@echo "  ./zkplex --help"
	@echo "  ./zkplex --circuit \"age >= 18\" --secret age:25 --prove"

# Start Play dev server with WASM from host
play-dev:
	@echo "Starting Play dev server..."
	@if [ ! -d "pkg" ]; then \
		echo "Error: ./pkg directory not found. Run 'make wasm-build' first."; \
		exit 1; \
	fi
	docker build -f Dockerfile.play -t zkplex-play .
	docker run -d \
		--name zkplex-play \
		-p 5173:5173 \
		-v $(PWD)/pkg:/app/pkg:ro \
		-v $(PWD)/play/src:/app/src:ro \
		zkplex-play
	@echo "✓ Play dev server started at http://localhost:5173"
	@echo "  WASM artifacts mounted from ./pkg"
	@echo "  Source code mounted from ./play/src"
	@echo ""
	@echo "To view logs: docker logs -f zkplex-play"
	@echo "To stop:      make play-stop"

# Stop Play container
play-stop:
	@echo "Stopping Play container..."
	-docker stop zkplex-play
	-docker rm zkplex-play
	@echo "✓ Play stopped"

# Restart Play container (stop + start)
play-restart: play-stop play-dev
	@echo "✓ Play restarted successfully!"

# Build Play for production
play-build:
	@echo "Building Play for production..."
	@if [ ! -d "pkg" ]; then \
		echo "Error: ./pkg directory not found. Run 'make wasm-build' first."; \
		exit 1; \
	fi
	@BUILD_ID="build-$$(date +%Y%m%d-%H%M%S)"; \
	echo "Build ID: $$BUILD_ID"; \
	echo "Building Play with npm..."; \
	cd play && BUILD_ID=$$BUILD_ID npm install && BUILD_ID=$$BUILD_ID npm run build
	@echo "✓ Play production build complete!"
	@echo "  Output: play/dist/"
	@echo ""
	@echo "To preview: cd play && npm run preview"

# Clean up all Docker artifacts
clean:
	@echo "Cleaning up Docker artifacts..."
	-docker stop zkplex-play
	-docker rm zkplex-play zkplex-wasm-temp
	-docker rmi zkplex-wasm-builder zkplex-cli zkplex-play
	@echo "✓ Docker artifacts cleaned"

# Clean only WASM artifacts
clean-wasm:
	@echo "Cleaning WASM artifacts..."
	-docker rm zkplex-wasm-temp
	-docker rmi zkplex-wasm-builder
	rm -rf pkg
	@echo "✓ WASM artifacts cleaned"

# Clean only CLI artifacts
clean-cli:
	@echo "Cleaning CLI artifacts..."
	-docker rmi zkplex-cli
	@echo "✓ CLI artifacts cleaned"

# Clean Play build artifacts
clean-play:
	@echo "Cleaning Play build artifacts..."
	rm -rf play/dist
	@echo "✓ Play build artifacts cleaned"

# Build everything from scratch
all: clean wasm-build play-dev
	@echo "✓ Full build complete!"