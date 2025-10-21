# ZKPlex Web Interface

Web interface for creating and verifying zero-knowledge proofs using Zircon circuits.

## Features

- Create zero-knowledge proofs through web interface
- Verify proofs
- Support for Zircon circuits
- Work with public and private signals
- Interact with zkplex-core via WASM API

## Technologies

- **React 19** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **shadcn/ui** - UI components
- **zkplex-core WASM** - ZK proof engine

## Installation and Running

### Quick Start (Using Makefile)

From the **project root directory**, use these commands:

```bash
# Build WASM module
make wasm-build

# Option 1: Run development server in Docker
make play-dev
# Access at http://localhost:5173
# Stop with: make play-stop

# Option 2: Build for production
make play-build
# Output: play/dist/
# Preview with: cd play && npm run preview
```

### Manual Setup (Without Docker)

#### 1. Build WASM Module

From the project root:

```bash
make wasm-build
# Or manually:
# wasm-pack build --target web
```

This creates a `pkg/` directory with the WASM module.

#### 2. Install Dependencies

```bash
cd play
npm install
```

#### 3. Run Development Server

```bash
npm run dev
```

The application will be available at `http://localhost:5173`

#### 4. Build for Production

```bash
npm run build
```

Built files will be in the `dist/` directory.

#### 5. Preview Production Build

```bash
npm run preview
```

### Available Make Commands

From the project root:

| Command | Description |
|---------|-------------|
| `make wasm-build` | Build WASM module and copy to ./pkg/ |
| `make play-build` | Build Play for production |
| `make play-dev` | Run Play dev server in Docker |
| `make play-stop` | Stop Play Docker container |
| `make clean-play` | Remove Play build artifacts (dist/) |
| `make clean-wasm` | Remove WASM artifacts |

### Full Build Workflow

```bash
# Complete build from scratch
make wasm-build && make play-build

# Or for Docker development
make wasm-build && make play-dev
```

## Usage

### Creating a Proof

1. Go to the "Create Proof" tab
2. Enter a circuit in Zircon format (e.g., `(A + B) > C`)
3. Add signals with their values
4. Mark public signals with the "Public" checkbox
5. Click "Generate Proof"
6. The proof will be displayed along with public signals and verification context

### Verifying a Proof

1. Go to the "Verify Proof" tab
2. Paste the proof data:
   - Proof (Base85)
   - Verification Context (Base85)
   - Public Signals (name-value pairs)
3. Click "Verify Proof"
4. The verification result will be displayed

## Circuit Examples

```
# Simple comparison
age >= 18

# Arithmetic
(A + B) * C > D

# Boolean operators
balance > 100 && balance < 10000

# Complex expressions
(income * 12) >= (expenses * 12 + savings)
```

## Supported Operators

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
- Boolean: `&&`, `||`

## Project Structure

```
play/
├── src/
│   ├── components/
│   │   ├── ui/              # shadcn/ui components
│   │   ├── ProofCreator.tsx # Proof creation component
│   │   └── ProofVerifier.tsx# Proof verification component
│   ├── hooks/
│   │   └── useZKPlex.ts     # Hook for WASM API
│   ├── lib/
│   │   └── utils.ts         # Utilities
│   ├── App.tsx              # Main component
│   ├── main.tsx             # Entry point
│   └── index.css            # Global styles
├── index.html
├── vite.config.ts
├── tailwind.config.js
└── package.json
```

## Development

### Adding New UI Components

shadcn/ui components are located in `src/components/ui/`. To add new ones:

1. Create a component file in `src/components/ui/`
2. Use the `cn()` utility from `lib/utils.ts` to merge classes
3. Follow the structure of existing components

### Working with WASM API

The `useZKPlex` hook provides access to the WASM API:

```typescript
const { prove, verify, loading, ready } = useZKPlex();

// Create proof
const result = await prove({
  circuit: "(A + B) > C",
  signals: {
    A: { value: "10", public: false },
    B: { value: "20", public: false },
    C: { value: "25", public: true }
  }
});

// Result contains:
// - proof: Base85-encoded proof
// - verify_context: Base85-encoded verification context
// - public_signals: Record<string, PublicSignal>
// - debug?: { circuit, k, strategy, secret_signals, warnings }

// Verify
const verification = await verify({
  proof: result.proof,
  verify_context: result.verify_context,
  public_signals: result.public_signals
});
```

## License

MIT OR Apache-2.0