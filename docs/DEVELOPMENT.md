# Development Guide

This guide covers development workflows and best practices for ZKPlex.

## WASM Cache Busting

ZKPlex implements comprehensive cache busting to ensure browsers always load the latest WASM version during development and after production deployments.

### How It Works

1. **Build ID Generation**
   - Each WASM build generates a unique `BUILD_ID` based on timestamp
   - Format: `build-YYYYMMDD-HHMMSS` (e.g., `build-20250121-143052`)
   - Stored in `pkg/build-info.json` alongside WASM artifacts

2. **Development Mode**
   - HTTP headers prevent caching: `Cache-Control: no-store, no-cache`
   - WASM module excluded from Vite's optimization cache
   - BUILD_ID logged on every page load for debugging

3. **Production Mode**
   - Asset files include content hash: `[name]-[hash][extname]`
   - BUILD_ID embedded in application at build time
   - Version information logged to browser console

### Build Info File

After building WASM, `pkg/build-info.json` contains:

```json
{
  "buildId": "build-20250121-143052",
  "timestamp": 1737471052
}
```

### Console Output

When loading the application, check browser console for:

```
🔄 Loading WASM
   Build ID: build-20250121-143052
   Built at: 1/21/2025, 2:30:52 PM
   Cache version: build-20250121-143052
```

### Build Commands

All build commands automatically handle BUILD_ID:

```bash
# Build WASM with BUILD_ID
make wasm-build

# Build Play with BUILD_ID
make play-build

# Build CLI with BUILD_ID
make cli-build
```

### Forcing Cache Refresh

If you encounter caching issues:

1. **Development**:
   ```bash
   make play-restart  # Restart dev server
   ```

2. **Production**:
   - Hard refresh: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
   - Clear site data in browser DevTools
   - New WASM build automatically gets new hash

### Implementation Details

**Files involved:**
- `Makefile` - Generates BUILD_ID and creates build-info.json
- `play/vite.config.ts` - Configures no-cache headers and build hashing
- `play/src/lib/zkplex-wasm.ts` - Loads and displays build info
- `src/wasm/bindings.rs` - Logs WASM function calls for debugging

**Vite Configuration:**
```typescript
{
  // Pass BUILD_ID to client
  define: {
    'import.meta.env.VITE_BUILD_ID': JSON.stringify(process.env.BUILD_ID)
  },

  // No-cache headers for .wasm files
  plugins: [{
    name: 'wasm-no-cache',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url?.endsWith('.wasm')) {
          res.setHeader('Cache-Control', 'no-store, no-cache');
        }
        next();
      });
    }
  }]
}
```

## Development Workflow

### Making Changes to WASM

1. Edit Rust source code in `src/`
2. Rebuild WASM:
   ```bash
   make wasm-build
   ```
3. Restart Play:
   ```bash
   make play-restart
   ```
4. Verify new BUILD_ID in browser console

### Making Changes to Web UI

1. Edit TypeScript/React code in `play/src/`
2. Changes hot-reload automatically (HMR)
3. No rebuild needed for UI-only changes

### Testing Production Build

```bash
# Build everything
make wasm-build
make play-build

# Preview production build
cd play && npm run preview
```

## Debugging

### WASM Function Logging

All WASM API calls log to browser console:

```javascript
// prove()
🔍 WASM prove() received JSON: {...}
🔍 Parsed request - circuit count: 1, signals count: 3
✅ Proof generated successfully

// verify()
🔍 WASM verify() called
✅ Proof is VALID

// estimate()
🔍 WASM estimate() called
✅ Circuit estimation completed
🔍 k = 11, estimated rows = 1024
```

### Viewing Build Information

Check browser console after page load:

```
🔄 Loading WASM
   Build ID: build-20250121-143052
   Built at: 1/21/2025, 2:30:52 PM
   Cache version: build-20250121-143052
🚀 zkplex-core WASM loaded: 0.1.0 (build-20250121-143052)
```

### Common Issues

**Old WASM still loading:**
1. Check BUILD_ID in console
2. Hard refresh browser (`Ctrl+Shift+R`)
3. Clear browser cache
4. Verify `pkg/build-info.json` was updated
5. Restart Play server: `make play-restart`

**WASM build succeeded but changes not visible:**
1. Verify you rebuilt WASM: `make wasm-build`
2. Check timestamp in `pkg/build-info.json`
3. Restart Play: `make play-restart`
4. Check browser console for new BUILD_ID

## Performance Monitoring

### Timing Information

ProofSuccessModal displays:
- **Proof Size** - Size of generated proof
- **Proving Time** - Time to generate proof
- **Circuit Size (k)** - Circuit parameter
- **Output** - Computed result value

### Console Logging

Enable detailed timing:
```javascript
console.time('prove');
const result = await prove(request);
console.timeEnd('prove');
```

## See Also

- [Docker Guide](DOCKER.md) - Docker build and deployment
- [CLI Reference](CLI.md) - CLI commands and options
- [WASM API](WASM_API.md) - JavaScript/TypeScript API
- [Architecture](ARCHITECTURE.md) - System architecture