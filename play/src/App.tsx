import { useState } from 'react';
import { ProofCreator } from './components/ProofCreator';
import { ProofVerifier } from './components/ProofVerifier';
import type { ProveResponse } from './hooks/useZKPlex';

function App() {
  const [activeTab, setActiveTab] = useState<'create' | 'verify'>('create');
  const [lastProof, setLastProof] = useState<{ proof: ProveResponse; circuit: string } | null>(null);
  const [createdProof, setCreatedProof] = useState<ProveResponse | null>(null);

  const handleProofCreated = (proof: ProveResponse, circuit: string) => {
    setLastProof({ proof, circuit });
    setCreatedProof(proof);
  };

  const handleSwitchToVerify = () => {
    setActiveTab('verify');
  };

  const handleNavigateToVerify = (proofData: { proof: string; verify_context: string; public_signals: any }) => {
    // Convert proof data to ProveResponse format for ProofVerifier
    const proveResponse: ProveResponse = {
      version: 1,  // Current proof format version
      proof: proofData.proof,
      verify_context: proofData.verify_context,
      public_signals: proofData.public_signals,
    };
    setLastProof({ proof: proveResponse, circuit: '' });
    setActiveTab('verify');
  };

  return (
    <div className="min-h-screen relative" style={{
      background: 'linear-gradient(135deg, #f8fafc 0%, #f1f5f9 25%, #e2e8f0 50%, #cbd5e1 75%, #94a3b8 100%)'
    }}>
      {/* SVG Filter for Liquid Glass Effect */}
      <svg style={{ position: 'absolute', width: 0, height: 0 }}>
        <defs>
          <filter id="glass-distortion">
            <feTurbulence type="turbulence" baseFrequency="0.05 0.05" numOctaves="1" result="turbulence" seed="2" />
            <feDisplacementMap in="SourceGraphic" in2="turbulence" scale="10" xChannelSelector="R" yChannelSelector="G" />
          </filter>
        </defs>
      </svg>

      <div className="container mx-auto py-8 px-4">
        {/* Header */}
        <header className="mb-8 max-w-4xl mx-auto">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="pl-4 pr-4 h-12 rounded-md flex items-center justify-center text-primary-foreground font-bold text-xl relative bg-primary text-primary-foreground">
                <span style={{ position: 'relative', zIndex: 1 }}>ZKPlex</span>
              </div>
              <div>
                <h1 className="text-2xl font-semibold">Play</h1>
                <p className="text-sm text-muted-foreground">
                  Create and verify zero-knowledge proofs using Zircon circuits
                </p>
              </div>
            </div>

            {/* Tab Navigation */}
            <div className="inline-flex rounded-md border bg-muted p-1">
              <button
                onClick={() => setActiveTab('create')}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === 'create'
                    ? 'bg-primary text-primary-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                Create Proof
              </button>
              <button
                onClick={handleSwitchToVerify}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === 'verify'
                    ? 'bg-primary text-primary-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                Verify Proof
              </button>
            </div>
          </div>
        </header>

        {/* Content */}
        <div className="max-w-4xl mx-auto">
          <div style={{ display: activeTab === 'create' ? 'block' : 'none' }}>
            <ProofCreator
              onProofCreated={handleProofCreated}
              onNavigateToVerify={handleNavigateToVerify}
              initialResult={createdProof}
            />
          </div>
          <div style={{ display: activeTab === 'verify' ? 'block' : 'none' }}>
            <ProofVerifier
              initialProof={lastProof?.proof}
              initialCircuit={lastProof?.circuit}
            />
          </div>
        </div>

        {/* Footer */}
        <footer className="mt-12 text-center text-sm text-muted-foreground">
          <p>
            Built with ZKPlex Core • Powered by Halo2 & WASM
          </p>
          <p className="mt-2">
            <a
              href="https://github.com/zkplex/zkplex-core"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-foreground transition-colors underline"
            >
              View on GitHub
            </a>
          </p>
        </footer>
      </div>
    </div>
  );
}

export default App;