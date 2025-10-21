import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Textarea } from './ui/textarea';
import { Checkbox } from './ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';
import { Loader2, Plus, Trash2, AlertCircle, Key, Copy, Check } from 'lucide-react';
import { useZKPlex, type ProveResponse } from '../contexts/ZKPlexContext';

// Signal type definition for ProofCreator
interface Signal {
  value?: string;
  encoding?: 'decimal' | 'hex' | 'base58' | 'base64' | 'base85';
  public: boolean;
}
import { CircuitGraph } from './CircuitGraph';
import { ProofSuccessModal } from './ProofSuccessModal';
import bs58 from 'bs58';

// Helper to get current statement text based on cursor position
function getStatementAtCursor(fullText: string, cursorLine: number): string {
  const lines = fullText.split('\n');
  const statements: string[] = [];
  const lineToStatementMap: number[] = [];

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
    const line = lines[lineIndex].trim();
    if (!line) {
      lineToStatementMap[lineIndex] = Math.max(0, statements.length - 1);
      continue;
    }
    const lineParts = line.split(';').map(p => p.trim()).filter(p => p.length > 0);
    if (lineParts.length === 0) {
      lineToStatementMap[lineIndex] = Math.max(0, statements.length - 1);
      continue;
    }
    lineToStatementMap[lineIndex] = statements.length;
    statements.push(...lineParts);
  }

  const targetStatementIndex = lineToStatementMap[cursorLine] ?? 0;
  return statements[targetStatementIndex] || fullText;
}

interface SignalInput {
  name: string;
  value: string;
  isPublic: boolean;
  isOutput: boolean;
  encoding: 'decimal' | 'hex' | 'base58' | 'base64' | 'base85';
}

interface ProofCreatorProps {
  onProofCreated?: (proof: ProveResponse, circuit: string) => void;
  onNavigateToVerify?: (proofData: { proof: string; verify_context: string; public_signals: any }) => void;
  initialResult?: ProveResponse | null;
}

export function ProofCreator({ onProofCreated, onNavigateToVerify, initialResult }: ProofCreatorProps) {
  const { prove, loading: wasmLoading, ready } = useZKPlex();
  const [circuit, setCircuit] = useState('(A + B) > C');
  const [preprocess, setPreprocess] = useState('');
  const [signals, setSignals] = useState<SignalInput[]>([
    { name: 'A', value: '10', isPublic: false, isOutput: false, encoding: 'decimal' },
    { name: 'B', value: '20', isPublic: true, isOutput: false, encoding: 'decimal' },
    { name: 'C', value: '25', isPublic: false, isOutput: false, encoding: 'decimal' },
    { name: 'result', value: '', isPublic: true, isOutput: true, encoding: 'decimal' },
  ]);
  const [proving, setProving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ProveResponse | null>(initialResult || null);
  const [provingTime, setProvingTime] = useState<number>(0);
  const [strategy, setStrategy] = useState<'auto' | 'lookup' | 'bitd' | 'boolean'>('auto');
  const [solanaKeyBase58, setSolanaKeyBase58] = useState<string>('');
  const [copiedBase58, setCopiedBase58] = useState(false);
  const [preprocessCursorLine, setPreprocessCursorLine] = useState(0);
  const [circuitCursorLine, setCircuitCursorLine] = useState(0);
  const [preprocessFullscreen, setPreprocessFullscreen] = useState(false);
  const [circuitFullscreen, setCircuitFullscreen] = useState(false);
  const [showSuccessModal, setShowSuccessModal] = useState(false);

  const addSignal = () => {
    setSignals([...signals, { name: '', value: '', isPublic: false, isOutput: false, encoding: 'decimal' }]);
  };

  const removeSignal = (index: number) => {
    setSignals(signals.filter((_, i) => i !== index));
  };

  const updateSignal = (index: number, field: keyof SignalInput, value: string | boolean) => {
    const newSignals = [...signals];
    newSignals[index] = { ...newSignals[index], [field]: value };
    setSignals(newSignals);
  };

  const generateSolanaKey = () => {
    // Generate a random 32-byte array for Solana keypair
    const keypair = new Uint8Array(32);
    crypto.getRandomValues(keypair);

    // Convert to base58
    const keyBase58 = bs58.encode(keypair);

    setSolanaKeyBase58(keyBase58);
    setCopiedBase58(false);
  };

  const copyBase58 = async () => {
    if (solanaKeyBase58) {
      await navigator.clipboard.writeText(solanaKeyBase58);
      setCopiedBase58(true);
      setTimeout(() => setCopiedBase58(false), 2000);
    }
  };

  const downloadProof = () => {
    if (!result) return;

    // Create JSON with proof data for download (same format as CLI output)
    const proofData = {
      version: result.version,
      proof: result.proof,
      verify_context: result.verify_context,
      public_signals: result.public_signals,
      debug: result.debug,
    };

    const blob = new Blob([JSON.stringify(proofData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'zkplex-proof.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleNavigateToVerify = () => {
    if (!result || !onNavigateToVerify) return;

    const proofData = {
      proof: result.proof,
      verify_context: result.verify_context,
      public_signals: result.public_signals,
    };

    setShowSuccessModal(false);
    onNavigateToVerify(proofData);
  };

  // Helper function to get the line number from cursor position
  const getCursorLine = (text: string, cursorPosition: number): number => {
    const textBeforeCursor = text.substring(0, cursorPosition);
    const lines = textBeforeCursor.split('\n');
    return lines.length - 1;
  };

  // Handler for preprocess textarea cursor changes
  const handlePreprocessCursorChange = (e: React.SyntheticEvent<HTMLTextAreaElement>) => {
    const textarea = e.target as HTMLTextAreaElement;
    const lineNumber = getCursorLine(preprocess, textarea.selectionStart);
    setPreprocessCursorLine(lineNumber);
  };

  // Handler for circuit textarea cursor changes
  const handleCircuitCursorChange = (e: React.SyntheticEvent<HTMLTextAreaElement>) => {
    const textarea = e.target as HTMLTextAreaElement;
    const lineNumber = getCursorLine(circuit, textarea.selectionStart);
    setCircuitCursorLine(lineNumber);
  };

  const handleProve = async () => {
    setError(null);
    setResult(null);
    setProving(true);

    // Use setTimeout to ensure the spinner renders before heavy computation
    await new Promise(resolve => setTimeout(resolve, 50));

    const startTime = performance.now();

    try {
      // Validate inputs
      if (!circuit.trim()) {
        throw new Error('Circuit cannot be empty');
      }

      const signalMap: Record<string, Signal> = {};
      for (const sig of signals) {
        if (!sig.name.trim()) {
          throw new Error('All signals must have a name');
        }
        // Output signals can have empty values (they will be computed)
        if (!sig.isOutput && !sig.value.trim()) {
          throw new Error(`Signal ${sig.name} must have a value (or mark it as Output)`);
        }

        // For output signals, don't send value (it will be computed)
        // For other signals, send the value
        if (sig.isOutput) {
          signalMap[sig.name] = {
            public: sig.isPublic,
          };
        } else {
          signalMap[sig.name] = {
            value: sig.value,
            public: sig.isPublic,
            encoding: sig.encoding,
          };
        }
      }

      // Split preprocess string into separate statements
      const preprocessStatements = preprocess.split(';')
        .map(s => s.trim())
        .filter(s => s.length > 0);

      // Split circuit string into separate statements
      const circuitStatements = circuit.split(';')
        .map(s => s.trim())
        .filter(s => s.length > 0);

      // Generate proof
      const proveRequest = {
        preprocess: preprocessStatements,
        circuit: circuitStatements,
        signals: signalMap,
        strategy,
      };

      const proof = await prove(proveRequest);


      const endTime = performance.now();
      setProvingTime(endTime - startTime);

      setResult(proof);
      setShowSuccessModal(true);
      if (onProofCreated) {
        onProofCreated(proof, circuit);
      }
    } catch (err) {
      console.error('Proof generation error:', err);
      // Extract detailed error message
      let errorMessage = 'Failed to generate proof';
      if (err instanceof Error) {
        errorMessage = err.message;
        // If there's a stack trace, log it for debugging
        if (err.stack) {
          console.error('Error stack:', err.stack);
        }
      } else if (typeof err === 'string') {
        errorMessage = err;
      }
      setError(errorMessage);
    } finally {
      setProving(false);
    }
  };

  if (wasmLoading) {
    return (
      <Card className="rounded-md border">
        <CardContent className="flex items-center justify-center py-8">
          <Loader2 className="h-8 w-8 animate-spin" />
          <span className="ml-2">Loading ZKPlex WASM...</span>
        </CardContent>
      </Card>
    );
  }

  if (!ready) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          Failed to initialize ZKPlex WASM module. Please refresh the page.
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="relative">
      {proving && (
        <div className="absolute inset-0 bg-white/10 dark:bg-gray-900/10 backdrop-blur-xl z-50 flex items-center justify-center rounded-md">
          <div className="flex flex-col items-center gap-3 bg-white/10 dark:bg-gray-900/10 backdrop-blur-xl p-8 rounded-md border border-white/20 dark:border-gray-700/30 shadow-2xl" style={{
            boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
          }}>
            <Loader2 className="h-12 w-12 animate-spin text-primary" />
            <p className="text-lg font-medium">Generating Proof...</p>
            <p className="text-sm text-muted-foreground ">This may take a few moments</p>
          </div>
        </div>
      )}
    <Card className="rounded-lg border" style={{
      boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)'
    }}>
      <CardHeader>
        <CardTitle>Create Zero-Knowledge Proof</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Preprocess Input with Graph */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2 p-4 border border-border border-1 border-purple-300 dark:border-purple-700 rounded-md bg-purple-50/30 dark:bg-purple-950/10">
            <Label htmlFor="preprocess" className="text-purple-700 dark:text-purple-400 font-medium">
              Preprocess (Optional)
            </Label>
            <Textarea
              id="preprocess"
              placeholder="e.g., hash<==sha256(A{%x}|B{%x})"
              value={preprocess}
              onChange={(e) => setPreprocess(e.target.value)}
              onClick={handlePreprocessCursorChange}
              onKeyUp={handlePreprocessCursorChange}
              onSelect={handlePreprocessCursorChange}
              className="font-mono text-sm bg-white dark:bg-gray-950 border-purple-200 dark:border-purple-800/50 focus-visible:ring-purple-500 rounded-md"
              rows={6}
            />
            <p className="text-xs text-muted-foreground">
              Hash functions: sha1, sha256, sha512, sha3_256, sha3_512, md5, crc32, blake2b, blake3, keccak256, ripemd160. Separate statements with newlines or semicolons.
            </p>
          </div>

          {/* Preprocess Visualization */}
          <div className="relative p-4 border border-border border-1 border-purple-300 dark:border-purple-700 rounded-md bg-white dark:bg-gray-950">
            <div className="flex items-center justify-between mb-2">
              <Label className="text-purple-700 dark:text-purple-400 font-medium">
                Preprocess Graph
              </Label>
            </div>
            <div className="mt-2">
              <CircuitGraph
                circuit={getStatementAtCursor(preprocess, preprocessCursorLine)}
                containerWidth={380}
                containerHeight={256}
                onClick={() => setPreprocessFullscreen(true)}
              />
            </div>
          </div>
        </div>

        {/* Circuit Input with Graph */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2 p-4 border border-border border-1 border-blue-300 dark:border-blue-700 rounded-md bg-blue-50 dark:bg-blue-950/20">
            <Label htmlFor="circuit" className="text-blue-700 dark:text-blue-400 font-medium">
              Circuit
            </Label>
            <Textarea
              id="circuit"
              placeholder="e.g., (A + B) > C"
              value={circuit}
              onChange={(e) => setCircuit(e.target.value)}
              onClick={handleCircuitCursorChange}
              onKeyUp={handleCircuitCursorChange}
              onSelect={handleCircuitCursorChange}
              className="font-mono text-sm bg-white dark:bg-gray-950 border-blue-200 dark:border-blue-800/50 focus-visible:ring-blue-500 rounded-md"
              rows={6}
            />
            <p className="text-xs text-muted-foreground">
              Enter circuit statements. Separate multiple statements with newlines or semicolons.
            </p>
          </div>

          {/* Circuit Visualization */}
          <div className="relative p-4 border border-border border-1 border-blue-300 dark:border-blue-700 rounded-md bg-white dark:bg-gray-950">
            <div className="flex items-center justify-between mb-2">
              <Label className="text-blue-700 dark:text-blue-400 font-medium">
                Circuit Graph
              </Label>
            </div>
            <div className="mt-2">
              <CircuitGraph
                circuit={getStatementAtCursor(circuit, circuitCursorLine)}
                containerWidth={380}
                containerHeight={256}
                onClick={() => setCircuitFullscreen(true)}
              />
            </div>
          </div>
        </div>

        {/* Preprocess Fullscreen Modal */}
        {preprocessFullscreen && (
          <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-8" onClick={() => setPreprocessFullscreen(false)}>
            <div className="bg-white dark:bg-gray-950 rounded-lg shadow-2xl p-[30px]" onClick={(e) => e.stopPropagation()}>
              <div className="flex flex-col items-center justify-center gap-6">
                <CircuitGraph
                  circuit={getStatementAtCursor(preprocess, preprocessCursorLine)}
                  containerWidth={760}
                  containerHeight={512}
                />
                {/* Statement text */}
                <div className="px-4 text-center">
                  <span className="text-sm font-mono text-muted-foreground">
                    {getStatementAtCursor(preprocess, preprocessCursorLine)}
                  </span>
                </div>
                {/* Legend */}
                <div className="flex flex-wrap gap-4 text-xs justify-center">
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded-full border-2 border-green-500 bg-transparent"></div>
                    <span>Output</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded-full border-2 border-blue-500 bg-transparent"></div>
                    <span>Operation</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded border-2 border-purple-500 bg-transparent"></div>
                    <span>Variable</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded border-2 border-slate-400 bg-transparent"></div>
                    <span>Constant</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Circuit Fullscreen Modal */}
        {circuitFullscreen && (
          <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-8" onClick={() => setCircuitFullscreen(false)}>
            <div className="bg-white dark:bg-gray-950 rounded-lg shadow-2xl p-[30px]" onClick={(e) => e.stopPropagation()}>
              <div className="flex flex-col items-center justify-center gap-6">
                <CircuitGraph
                  circuit={getStatementAtCursor(circuit, circuitCursorLine)}
                  containerWidth={760}
                  containerHeight={512}
                />
                {/* Statement text */}
                <div className="px-4 text-center">
                  <span className="text-sm font-mono text-muted-foreground">
                    {getStatementAtCursor(circuit, circuitCursorLine)}
                  </span>
                </div>
                {/* Legend */}
                <div className="flex flex-wrap gap-4 text-xs justify-center">
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded-full border-2 border-green-500 bg-transparent"></div>
                    <span>Output</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded-full border-2 border-blue-500 bg-transparent"></div>
                    <span>Operation</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded border-2 border-purple-500 bg-transparent"></div>
                    <span>Variable</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded border-2 border-slate-400 bg-transparent"></div>
                    <span>Constant</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Proof Strategy and Solana Key Generator Row */}
        <div className="grid grid-cols-2 gap-4">
          {/* Proof Strategy Selector */}
          <div className="space-y-2 p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
            <Label htmlFor="strategy">Proof Strategy</Label>
            <Select value={strategy} onValueChange={(value) => setStrategy(value as 'auto' | 'lookup' | 'bitd' | 'boolean')}>
              <SelectTrigger id="strategy">
                <SelectValue placeholder="Select strategy" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">Auto (Balanced)</SelectItem>
                <SelectItem value="boolean">Boolean (Base Strategy)</SelectItem>
                <SelectItem value="lookup">Lookup (Faster Proving)</SelectItem>
                <SelectItem value="bitd">Bit Decomposition (Smaller Proofs)</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {strategy === 'auto' && (
                <>Balanced strategy, supports all operators (~30-40 KB proofs).</>
              )}
              {strategy === 'boolean' && (
                <>Boolean operations (~30-40 KB). Supports: +, -, *, /, ==, !=, AND, OR, NOT. Cannot use: &gt;, &lt;, &gt;=, &lt;=.</>
              )}
              {strategy === 'lookup' && (
                <>Faster proving with lookup tables (~30-40 KB). Full support.</>
              )}
              {strategy === 'bitd' && (
                <>Bit decomposition (~30-40 KB). Full support.</>
              )}
            </p>
          </div>

          {/* Solana Key Generator */}
          <div className="space-y-3 p-4 border rounded-md bg-muted/30">
            <div className="flex items-center justify-between">
              <Label>Solana Key Generator</Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={generateSolanaKey}
                className="rounded-md"
              >
                <Key className="h-4 w-4 mr-1" />
                Generate Key
              </Button>
            </div>
            {solanaKeyBase58 && (
              <div className="space-y-2">
                <Label className="text-xs">Base58 (Solana format) <span className="text-xs text-muted-foreground pl-11">32-byte random key for Solana</span></Label>
                <div className="flex items-center gap-2">
                  <Input
                    value={solanaKeyBase58}
                    readOnly
                    className="font-mono text-xs rounded-md"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    onClick={copyBase58}
                    title="Copy Base58"
                    className="rounded-md"
                  >
                    {copiedBase58 ? (
                      <Check className="h-4 w-4 text-green-500" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Input Signals */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label>Input Signals</Label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={addSignal}
              className="rounded-md"
            >
              <Plus className="h-4 w-4 mr-1" />
              Add Signal
            </Button>
          </div>

          <div className="border border-border rounded-md overflow-hidden border-1 border-gray-300 dark:border-gray-600">
            <table className="w-full">
              <thead className="bg-muted/50">
                <tr className="border-b border-border">
                  <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground w-24">Type</th>
                  <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground">Name</th>
                  <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground">Value</th>
                  <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground w-32">Encoding</th>
                  <th className="px-4 py-2 text-center text-sm font-normal text-muted-foreground w-16"></th>
                </tr>
              </thead>
              <tbody>
                {signals.filter(s => !s.isOutput).map((signal) => {
                  const originalIndex = signals.indexOf(signal);
                  return (
                    <tr key={originalIndex} className="border-b border-border last:border-b-0 hover:bg-muted/30">
                      <td className="px-4 py-2">
                        <div className="flex items-center gap-2">
                          <Checkbox
                            id={`signal-secret-${originalIndex}`}
                            checked={!signal.isPublic}
                            onCheckedChange={(checked) => updateSignal(originalIndex, 'isPublic', !checked)}
                            className="h-4 w-4 rounded-sm"
                          />
                          {!signal.isPublic ? (
                            <span className="inline-flex items-center px-2 py-1 rounded-md bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 text-xs font-medium cursor-pointer hover:bg-purple-200 dark:hover:bg-purple-900/50 transition-colors"
                              onClick={() => updateSignal(originalIndex, 'isPublic', true)}>
                              Secret
                            </span>
                          ) : (
                            <span className="inline-flex items-center px-2 py-1 rounded-md bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium cursor-pointer hover:bg-blue-200 dark:hover:bg-blue-900/50 transition-colors"
                              onClick={() => updateSignal(originalIndex, 'isPublic', false)}>
                              Public
                            </span>
                          )}
                        </div>
                      </td>
                      <td className="px-4 py-2">
                        <Input
                          placeholder="Signal name"
                          value={signal.name}
                          onChange={(e) => updateSignal(originalIndex, 'name', e.target.value)}
                          className="h-8 text-sm border border-gray-300 dark:border-gray-600 bg-white/50 dark:bg-gray-950/50 focus-visible:ring-1 rounded-md"
                        />
                      </td>
                      <td className="px-4 py-2">
                        <Input
                          placeholder="Value"
                          value={signal.value}
                          onChange={(e) => updateSignal(originalIndex, 'value', e.target.value)}
                          className="h-8 font-mono text-sm border border-gray-300 dark:border-gray-600 bg-white/50 dark:bg-gray-950/50 focus-visible:ring-1 rounded-md"
                        />
                      </td>
                      <td className="px-4 py-2">
                        <Select value={signal.encoding} onValueChange={(value) => updateSignal(originalIndex, 'encoding', value)}>
                          <SelectTrigger className="h-8 text-sm border border-gray-300 dark:border-gray-600 bg-white/50 dark:bg-gray-950/50 focus-visible:ring-1 rounded-md">
                            <SelectValue placeholder="Encoding" />
                          </SelectTrigger>
                          <SelectContent className="rounded-md">
                            <SelectItem value="decimal">Decimal</SelectItem>
                            <SelectItem value="hex">Hex</SelectItem>
                            <SelectItem value="base58">Base58</SelectItem>
                            <SelectItem value="base64">Base64</SelectItem>
                            <SelectItem value="base85">Base85</SelectItem>
                          </SelectContent>
                        </Select>
                      </td>
                      <td className="px-4 py-2 text-center">
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7 rounded-md"
                          onClick={() => removeSignal(originalIndex)}
                          disabled={signals.length === 1}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>

        {/* Output Signal */}
        {signals.find(s => s.isOutput) && (
          <div className="space-y-2">
            <Label>Output Signal</Label>
            <div className="border border-green-300 dark:border-green-700 rounded-md bg-green-50/30 dark:bg-green-950/10 p-4">
              {(() => {
                const outputSignal = signals.find(s => s.isOutput);
                const outputIndex = signals.indexOf(outputSignal!);
                return (
                  <div className="flex items-center gap-4">
                    <span className="inline-flex items-center px-2 py-1 rounded-md bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 text-xs font-medium flex-shrink-0">
                      Output
                    </span>
                    <Input
                      placeholder="Output signal name"
                      value={outputSignal!.name}
                      onChange={(e) => updateSignal(outputIndex, 'name', e.target.value)}
                      className="h-8 text-sm border border-green-300 dark:border-green-700 bg-white dark:bg-gray-950 rounded-md flex-1"
                    />
                    <Input
                      placeholder="Auto-computed"
                      value={outputSignal!.value}
                      disabled
                      className="h-8 font-mono text-sm border border-green-300 dark:border-green-700 bg-gray-100 dark:bg-gray-900 rounded-md opacity-60 flex-1"
                    />
                  </div>
                );
              })()}
            </div>
          </div>
        )}

        {/* Generate Button */}
        <div className="flex justify-center">
          <Button
            onClick={handleProve}
            disabled={proving}
            className="min-w-48 rounded-md"
            size="lg"
          >
            {proving ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Generating Proof...
              </>
            ) : (
              'Generate Proof'
            )}
          </Button>
        </div>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive" className="border-red-300 dark:border-red-800">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle className="text-lg font-semibold">Proof Generation Failed</AlertTitle>
            <AlertDescription className="mt-2">
              <div className="space-y-2">
                <p className="font-medium">Error Details:</p>
                <div className="p-3 bg-red-50 dark:bg-red-950/30 rounded-md border border-red-200 dark:border-red-800/50">
                  <pre className="text-xs font-mono whitespace-pre-wrap break-words text-red-900 dark:text-red-100">
                    {error}
                  </pre>
                </div>
                <p className="text-xs text-muted-foreground mt-2">
                  Please check your circuit syntax, signal values, and ensure all inputs are valid.
                </p>
              </div>
            </AlertDescription>
          </Alert>
        )}

        {/* Result Display */}
        {result && (
          <div className="space-y-4">
            <Alert className="border">
              <AlertTitle>Proof Generated Successfully!</AlertTitle>
              <AlertDescription>
                Your zero-knowledge proof has been generated successfully.
              </AlertDescription>
            </Alert>

            {/* Proof Statistics */}
            <div className="grid grid-cols-3 gap-4">
              <div className="p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
                <div className="text-2xl font-semibold">{(() => {
                  // Base85 encoded string size
                  const bytes = new Blob([result.proof]).size;
                  if (bytes < 1024) return `${bytes} B`;
                  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
                  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
                })()}</div>
                <p className="text-xs text-muted-foreground mt-1">Encoded size (Base85)</p>
              </div>
              <div className="p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
                <div className="text-2xl font-semibold">{(() => {
                  // Estimate decoded size (Base85 has ~25% overhead)
                  const stringBytes = new Blob([result.proof]).size;
                  const decodedBytes = Math.round(stringBytes / 1.25);
                  if (decodedBytes < 1024) return `${decodedBytes} B`;
                  if (decodedBytes < 1024 * 1024) return `${(decodedBytes / 1024).toFixed(2)} KB`;
                  return `${(decodedBytes / (1024 * 1024)).toFixed(2)} MB`;
                })()}</div>
                <p className="text-xs text-muted-foreground mt-1">Binary size (estimated)</p>
              </div>
              <div className="p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
                <div className="text-2xl font-semibold">{(provingTime / 1000).toFixed(2)}s</div>
                <p className="text-xs text-muted-foreground mt-1">Proving time</p>
              </div>
            </div>

            {/* Proof Size Info */}
            <Alert className="rounded-md border border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-950/20">
              <AlertCircle className="h-4 w-4 text-blue-600 dark:text-blue-400" />
              <AlertTitle className="text-blue-700 dark:text-blue-300">About Proof Size</AlertTitle>
              <AlertDescription className="text-xs text-blue-600 dark:text-blue-400">
                Halo2 proofs are ~30-40 KB for simple circuits (k={result.debug?.k}). The size comes from cryptographic commitments
                (advice columns, permutations, lookups) and IPA rounds. This is normal for Halo2's flexible architecture.
                Other systems like Groth16 (~200 bytes) or Plonk (~1-2 KB) are more compact but less flexible.
              </AlertDescription>
            </Alert>

            {result.debug?.warnings && result.debug.warnings.length > 0 && (
              <Alert className="rounded-md border">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Warnings</AlertTitle>
                <AlertDescription>
                  <ul className="list-disc list-inside space-y-1">
                    {result.debug.warnings.map((warning, i) => (
                      <li key={i} className="text-xs">{warning}</li>
                    ))}
                  </ul>
                </AlertDescription>
              </Alert>
            )}

            <div className="space-y-2 p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
              <Label>Proof (Base85)</Label>
              <Textarea
                value={result.proof}
                readOnly
                className="font-mono text-xs bg-white dark:bg-gray-950 rounded-md"
                rows={4}
              />
            </div>

            {/* Strategy and k Parameter */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2 p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
                <Label>Strategy</Label>
                <Input value={result.debug?.strategy || strategy} readOnly className="bg-white dark:bg-gray-950 rounded-md" />
                <p className="text-xs text-muted-foreground">
                  Range check strategy used for proof generation
                </p>
              </div>
              <div className="space-y-2 p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
                <Label>k Parameter</Label>
                <Input value={result.debug?.k} readOnly className="bg-white dark:bg-gray-950 rounded-md" />
                <p className="text-xs text-muted-foreground">
                  Circuit size parameter (2^k constraint rows)
                </p>
              </div>
            </div>

            {/* Public Signals Table */}
            <div className="space-y-2">
              <Label>Public Signals</Label>
              <div className="border rounded-md overflow-hidden">
                <table className="w-full">
                  <thead className="bg-muted/50">
                    <tr className="border-b border-border">
                      <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground w-24">Type</th>
                      <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground">Name</th>
                      <th className="px-4 py-2 text-left text-sm font-normal text-muted-foreground">Value</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(result.public_signals)
                      .filter(([name]) => !signals.find(s => s.isOutput && s.name === name))
                      .map(([name, publicSignal]) => (
                        <tr key={name} className="border-b border-border last:border-b-0 hover:bg-muted/30">
                          <td className="px-4 py-3">
                            <span className="inline-flex items-center px-2 py-1 rounded-md bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium">
                              Public
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <span className="text-sm font-medium">{name}</span>
                            {publicSignal.encoding && (
                              <span className="ml-2 text-xs text-muted-foreground">({publicSignal.encoding})</span>
                            )}
                          </td>
                          <td className="px-4 py-3">
                            <Input
                              value={publicSignal.value}
                              readOnly
                              className="h-8 font-mono text-sm border-0 bg-transparent focus-visible:ring-1 rounded-md"
                            />
                          </td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Output Signal Result */}
            {(() => {
              const outputSig = signals.find(s => s.isOutput);
              if (!outputSig) return null;
              const outputSignal = result.public_signals[outputSig.name];
              if (!outputSignal) return null;

              return (
                <div className="space-y-2">
                  <Label>Output Signal</Label>
                  <div className="border border-green-300 dark:border-green-700 rounded-md bg-green-50/30 dark:bg-green-950/10 p-4">
                    <div className="flex items-center gap-4">
                      <span className="inline-flex items-center px-2 py-1 rounded-md bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 text-xs font-medium flex-shrink-0">
                        Output
                      </span>
                      <span className="text-sm font-medium flex-shrink-0">{outputSig.name}</span>
                      <Input
                        value={outputSignal.value}
                        readOnly
                        className="h-8 font-mono text-sm border border-green-300 dark:border-green-700 bg-white dark:bg-gray-950 rounded-md flex-1"
                      />
                    </div>
                  </div>
                </div>
              );
            })()}
          </div>
        )}
      </CardContent>
    </Card>

    {/* Success Modal */}
    {showSuccessModal && result && (
      <ProofSuccessModal
        result={result}
        provingTime={provingTime}
        circuit={circuit}
        onClose={() => setShowSuccessModal(false)}
        onDownload={downloadProof}
        onNavigateToVerify={handleNavigateToVerify}
      />
    )}
    </div>
  );
}