# Docker Build Guide for ZKPlex

This guide explains how to build WASM and run Play using Docker.

## Prerequisites

- Docker installed and running
- Make (optional, but recommended)

## Quick Start

```bash
# 1. Build WASM in Docker and copy to ./pkg
make wasm-build

# 2. Start Play dev server
make play-dev

# 3. Open http://localhost:5173 in your browser
```

## Available Commands

### WASM Compilation

```bash
# Build WASM from source in Docker
make wasm-build
```

This will:
1. Build a Docker image with Rust and wasm-pack
2. Compile zkplex-core to WASM
3. Copy the WASM artifacts to `./pkg` on your host machine

**Output:** `./pkg/` directory with:
- `zkplex_core.js` - JavaScript bindings
- `zkplex_core_bg.wasm` - WebAssembly binary
- `zkplex_core.d.ts` - TypeScript definitions
- `package.json` - Package metadata

### Play Development

```bash
# Start Play dev server (requires ./pkg from wasm-build)
make play-dev

# View logs
docker logs -f zkplex-play

# Stop play
make play-stop
```

The Play dev server will:
- Mount `./pkg` (WASM artifacts) as read-only
- Mount `./play/src` (source code) as read-only for hot reload
- Expose port 5173 for access from host

**Access:** http://localhost:5173

### Cleanup

```bash
# Remove all Docker artifacts (containers and images)
make clean

# Remove only WASM-related artifacts
make clean-wasm
```

## Manual Docker Commands

If you prefer not to use Make:

### Build WASM manually

```bash
# Build image
docker build -f Dockerfile.wasm -t zkplex-wasm-builder .

# Create container
docker create --name zkplex-wasm-temp zkplex-wasm-builder

# Copy artifacts
docker cp zkplex-wasm-temp:/output ./pkg

# Cleanup
docker rm zkplex-wasm-temp
```

### Run Play manually

```bash
# Build image
docker build -f Dockerfile.play -t zkplex-play .

# Run container
docker run -d \
  --name zkplex-play \
  -p 5173:5173 \
  -v $(pwd)/pkg:/app/pkg:ro \
  -v $(pwd)/play/src:/app/src:ro \
  zkplex-play

# Stop container
docker stop zkplex-play
docker rm zkplex-play
```

## Architecture

### WASM Build Container

**Dockerfile:** `Dockerfile.wasm`

- Base: `rust:1.90-slim`
- Installs: wasm-pack, OpenSSL
- Compiles: zkplex-core → WASM
- Output: `/output` directory (copied to host `./pkg`)

**Why Docker?**
- Reproducible builds across different host environments
- No need to install Rust/wasm-pack on host
- Consistent Rust version (1.90)

### Play Container

**Dockerfile:** `Dockerfile.play`

- Base: `node:20-slim`
- Installs: npm dependencies
- Runs: Vite dev server on port 5173
- Mounts:
  - `./pkg` → `/app/pkg` (WASM artifacts, read-only)
  - `./play/src` → `/app/src` (source code, read-only for hot reload)

**Why Docker?**
- No need to install Node.js on host
- Isolated development environment
- Easy to share and reproduce

## Workflow

### Development Workflow

1. **Edit Rust code** in `src/`
2. **Rebuild WASM:** `make wasm-build`
3. **Restart Play:** `make play-stop && make play-dev`
4. **Edit Play code** in `play/src/` (hot reload works automatically)

### CI/CD Integration

```bash
# In CI pipeline
make wasm-build
# Artifacts are now in ./pkg, ready to publish or deploy
```

## Troubleshooting

### Error: "pkg directory not found"

**Solution:** Run `make wasm-build` first to compile WASM.

### Error: "Port 5173 already in use"

**Solution:**
```bash
# Stop existing container
make play-stop

# Or kill process using the port
lsof -ti:5173 | xargs kill -9
```

### WASM changes not reflected

**Solution:** Rebuild WASM and restart Play:
```bash
make wasm-build
make play-stop
make play-dev
```

### Source code changes not hot-reloading

**Cause:** Source directory not mounted correctly.

**Solution:** Ensure you're running from project root and `./play/src` exists.

## Performance Notes

- **WASM build time:** ~2-5 minutes (first build), ~30-60 seconds (incremental)
- **Play startup:** ~10-20 seconds
- **Hot reload:** Instant for Play code changes

## Comparison: Docker vs Local

| Aspect | Docker | Local |
|--------|--------|-------|
| **Setup** | Just Docker | Rust + wasm-pack + Node.js |
| **Build time** | Slightly slower | Faster |
| **Reproducibility** | 100% reproducible | Depends on host environment |
| **Portability** | Works everywhere | Requires manual setup |
| **CI/CD** | Perfect | Requires installing dependencies |

## Next Steps

- **Production deployment:** Use `./pkg` artifacts in your production app
- **npm package:** Publish `./pkg` to npm registry
- **Custom builds:** Modify `Dockerfile.wasm` for custom targets (nodejs, bundler, etc.)

## See Also

- [QUICKSTART.md](QUICKSTART.md) - Local development without Docker
- [README.md](README.md) - Project overview
- [play/README.md](play/README.md) - Play documentation