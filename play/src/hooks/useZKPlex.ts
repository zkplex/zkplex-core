// DEPRECATED: This hook is deprecated. Import from '../contexts/ZKPlexContext' instead.
// This file is kept for backwards compatibility only.

export { useZKPlex } from '../contexts/ZKPlexContext';
export type {
  ProveRequest,
  ProveResponse,
  VerifyRequest,
  VerifyResponse,
  PublicSignal
} from '../contexts/ZKPlexContext';

// Additional types that were previously here
export interface Signal {
  value?: string;  // Optional for output signals (will be computed)
  encoding?: 'decimal' | 'hex' | 'base58' | 'base64' | 'base85';
  public: boolean;
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