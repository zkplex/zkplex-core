import { createContext, useContext, useEffect, useState, ReactNode } from 'react';

// Types from zkplex-core
export interface Signal {
  value?: string;  // Optional for output signals (will be computed)
  encoding?: 'decimal' | 'hex' | 'base58' | 'base64' | 'base85';
  public: boolean;
}

export interface PublicSignal {
  value: string;
  encoding?: 'decimal' | 'hex' | 'base58' | 'base64' | 'base85';
}

export interface ProveRequest {
  preprocess: string[];
  circuit: string[];
  signals: Record<string, Signal>;
  strategy?: 'auto' | 'lookup' | 'bitd' | 'boolean';
}

export interface DebugInfo {
  preprocess: string[];
  circuit: string[];
  k: number;
  strategy: string;
  secret_signals: string[];
  output_signal?: string;
  warnings?: string[];
}

export interface ProveResponse {
  version: number;
  proof: string;
  verify_context: string;
  public_signals: Record<string, PublicSignal>;
  debug?: DebugInfo;
  strategy?: string;
  k?: number;
}

export interface VerifyRequest {
  version?: number;
  proof: string;
  verify_context: string;
  public_signals: Record<string, PublicSignal>;
}

export interface VerifyResponse {
  valid: boolean;
  error?: string;
}

interface ZKPlexWASM {
  prove: (request: string) => string;
  verify: (request: string) => string;
}

interface ZKPlexContextValue {
  prove: (request: ProveRequest) => Promise<ProveResponse>;
  verify: (request: VerifyRequest) => Promise<VerifyResponse>;
  loading: boolean;
  error: string | null;
  ready: boolean;
}

const ZKPlexContext = createContext<ZKPlexContextValue | null>(null);

// Singleton to prevent double initialization in StrictMode
let wasmSingleton: ZKPlexWASM | null = null;
let wasmInitPromise: Promise<ZKPlexWASM> | null = null;

export function ZKPlexProvider({ children }: { children: ReactNode }) {
  const [wasm, setWasm] = useState<ZKPlexWASM | null>(wasmSingleton);
  const [loading, setLoading] = useState(!wasmSingleton);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // If already initialized, skip
    if (wasmSingleton) {
      setWasm(wasmSingleton);
      setLoading(false);
      return;
    }

    // If initialization is in progress, wait for it
    if (wasmInitPromise) {
      wasmInitPromise.then(module => {
        setWasm(module);
        setLoading(false);
      }).catch(err => {
        console.error('Failed to initialize WASM:', err);
        setError(err instanceof Error ? err.message : 'Failed to initialize WASM');
        setLoading(false);
      });
      return;
    }

    // Start new initialization
    let mounted = true;

    async function initWasm() {
      try {
        // Import the WASM module wrapper
        const wasmModule = await import('@/lib/zkplex-wasm');
        await wasmModule.default();

        // Log version for debugging
        if (wasmModule.version) {
          const ver = wasmModule.version();
          console.log('🚀 zkplex-core WASM loaded:', ver);
        }

        // Store in singleton
        wasmSingleton = wasmModule;

        if (mounted) {
          setWasm(wasmModule);
          setLoading(false);
        }

        return wasmModule;
      } catch (err) {
        console.error('Failed to initialize WASM:', err);
        if (mounted) {
          setError(err instanceof Error ? err.message : 'Failed to initialize WASM');
          setLoading(false);
        }
        throw err;
      }
    }

    wasmInitPromise = initWasm();

    return () => {
      mounted = false;
    };
  }, []);

  const prove = async (request: ProveRequest): Promise<ProveResponse> => {
    if (!wasm) {
      throw new Error('WASM not initialized');
    }

    try {
      const response = wasm.prove(JSON.stringify(request));
      const parsed = JSON.parse(response);
      return parsed;
    } catch (err) {
      console.error('❌ WASM prove error:', err);
      throw new Error(err instanceof Error ? err.message : 'Proof generation failed');
    }
  };

  const verify = async (request: VerifyRequest): Promise<VerifyResponse> => {
    if (!wasm) {
      throw new Error('WASM not initialized');
    }

    try {
      const response = wasm.verify(JSON.stringify(request));
      const parsed = JSON.parse(response);
      return parsed;
    } catch (err) {
      console.error('❌ WASM verify error:', err);
      throw new Error(err instanceof Error ? err.message : 'Proof verification failed');
    }
  };

  const value: ZKPlexContextValue = {
    prove,
    verify,
    loading,
    error,
    ready: !loading && !error && wasm !== null,
  };

  return (
    <ZKPlexContext.Provider value={value}>
      {children}
    </ZKPlexContext.Provider>
  );
}

export function useZKPlex(): ZKPlexContextValue {
  const context = useContext(ZKPlexContext);
  if (!context) {
    throw new Error('useZKPlex must be used within ZKPlexProvider');
  }
  return context;
}