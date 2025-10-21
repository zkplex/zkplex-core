import { useState, useRef, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Textarea } from './ui/textarea';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';
import { Loader2, CheckCircle2, XCircle, AlertCircle, Plus, Trash2, Upload } from 'lucide-react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { useZKPlex, type ProveResponse, type PublicSignal } from '../contexts/ZKPlexContext';

interface SignalRow {
  name: string;
  value: string;
  isOutput?: boolean;
  encoding?: 'decimal' | 'hex' | 'base58' | 'base64' | 'base85';
}

interface ProofVerifierProps {
  initialProof?: ProveResponse;
  initialCircuit?: string;
}

export function ProofVerifier({ initialProof }: ProofVerifierProps) {
  const { verify, loading: wasmLoading, ready } = useZKPlex();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [proof, setProof] = useState(initialProof?.proof || '');
  const [verifyContext, setVerifyContext] = useState(initialProof?.verify_context || '');
  const [signalRows, setSignalRows] = useState<SignalRow[]>(
    initialProof
      ? Object.entries(initialProof.public_signals).map(([name, publicSignal]) => ({
          name,
          value: publicSignal.value,
          encoding: publicSignal.encoding
        }))
      : [{ name: 'C', value: '25' }]
  );
  const [verifying, setVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<{ valid: boolean; error?: string } | null>(null);
  const [uploadError, setUploadError] = useState<string | null>(null);

  // Update form when initialProof changes (e.g., when navigating from Create Proof)
  useEffect(() => {
    if (initialProof) {
      setProof(initialProof.proof);
      setVerifyContext(initialProof.verify_context);

      // Update signal rows from initial proof
      const entries = Object.entries(initialProof.public_signals);
      const newSignalRows: SignalRow[] = entries.map(([name, publicSignal], index) => {
        // Determine if this is the output signal
        // Output signal is typically the last one or named "result"/"output"
        const isOutput = index === entries.length - 1 ||
                        name.toLowerCase().includes('result') ||
                        name.toLowerCase().includes('output');

        return {
          name,
          value: publicSignal.value,
          encoding: publicSignal.encoding,
          isOutput
        };
      });

      setSignalRows(newSignalRows);
    }
  }, [initialProof]);

  const addSignalRow = () => {
    setSignalRows([...signalRows, { name: '', value: '' }]);
  };

  const removeSignalRow = (index: number) => {
    setSignalRows(signalRows.filter((_, i) => i !== index));
  };

  const updateSignalRow = (index: number, field: 'name' | 'value' | 'encoding', value: string | ('decimal' | 'hex' | 'base58' | 'base64' | 'base85' | undefined)) => {
    const newRows = [...signalRows];
    newRows[index] = { ...newRows[index], [field]: value };
    setSignalRows(newRows);
  };

  const handleUploadProof = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setUploadError(null);
    setError(null);
    setResult(null);

    try {
      const text = await file.text();
      const data = JSON.parse(text);

      // Validate JSON structure
      if (!data.proof || !data.verify_context || !data.public_signals) {
        throw new Error('Invalid proof file format. Missing required fields: proof, verify_context, or public_signals');
      }

      // Populate form fields
      setProof(data.proof);
      setVerifyContext(data.verify_context);

      // Populate signals table
      const entries = Object.entries(data.public_signals);
      const newSignalRows: SignalRow[] = entries.map(([name, signal]: [string, any], index) => {
        // Determine if this is the output signal
        // Output signal is typically the last one or named "result"/"output"
        const isOutput = index === entries.length - 1 ||
                        name.toLowerCase().includes('result') ||
                        name.toLowerCase().includes('output');

        return {
          name,
          value: signal.value,
          encoding: signal.encoding,
          isOutput
        };
      });
      setSignalRows(newSignalRows);

      // Note: output field is ignored (not used for verification)
    } catch (err) {
      setUploadError(err instanceof Error ? err.message : 'Failed to parse proof file');
    }

    // Reset file input
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const handleVerify = async () => {
    setError(null);
    setResult(null);
    setVerifying(true);

    // Use setTimeout to ensure the spinner renders before heavy computation
    await new Promise(resolve => setTimeout(resolve, 50));

    try {
      // Validate inputs
      if (!proof.trim()) {
        throw new Error('Proof cannot be empty');
      }
      if (!verifyContext.trim()) {
        throw new Error('Verification context cannot be empty');
      }

      // Build public signals from table rows with encoding information
      const parsedPublicSignals: Record<string, PublicSignal> = {};
      for (const row of signalRows) {
        if (!row.name.trim()) {
          throw new Error('All signal names must be filled');
        }
        if (!row.value.trim()) {
          throw new Error(`Signal ${row.name} must have a value`);
        }
        parsedPublicSignals[row.name] = {
          value: row.value,
          encoding: row.encoding,
        };
      }

      // Verify proof using new API with encoding support
      const verificationResult = await verify({
        version: 1,  // Current proof format version
        proof,
        verify_context: verifyContext,
        public_signals: parsedPublicSignals,
      });

      setResult(verificationResult);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to verify proof');
    } finally {
      setVerifying(false);
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
      <Alert variant="destructive" className="rounded-md">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          Failed to initialize ZKPlex WASM module. Please refresh the page.
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <Card className="relative rounded-md border" style={{
      boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)'
    }}>
      {verifying && (
        <div className="absolute inset-0 bg-white/10 dark:bg-gray-900/10 backdrop-blur-xl z-50 flex items-center justify-center rounded-md">
          <div className="flex flex-col items-center gap-3 bg-white/10 dark:bg-gray-900/10 backdrop-blur-xl p-8 rounded-md border border-white/20 dark:border-gray-700/30 shadow-2xl" style={{
            boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
          }}>
            <Loader2 className="h-12 w-12 animate-spin text-primary" />
            <p className="text-lg font-medium">Verifying Proof...</p>
            <p className="text-sm text-muted-foreground">Please wait</p>
          </div>
        </div>
      )}
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Verify Zero-Knowledge Proof</CardTitle>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleUploadProof}
            className="rounded-md"
          >
            <Upload className="h-4 w-4 mr-2" />
            Upload Proof
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Hidden file input */}
        <input
          ref={fileInputRef}
          type="file"
          accept=".json,application/json"
          onChange={handleFileChange}
          className="hidden"
        />

        {/* Upload Error Display */}
        {uploadError && (
          <Alert variant="destructive" className="rounded-md">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Upload Error</AlertTitle>
            <AlertDescription>{uploadError}</AlertDescription>
          </Alert>
        )}
        {/* Proof Input */}
        <div className="space-y-2 p-4 border border-border rounded-md bg-muted/30 border-1 border-gray-300 dark:border-gray-600">
          <Label htmlFor="proof">Proof (Base85)</Label>
          <Textarea
            id="proof"
            placeholder="Paste the proof data here"
            value={proof}
            onChange={(e) => setProof(e.target.value)}
            className="font-mono text-xs bg-white dark:bg-gray-950 rounded-md"
            rows={4}
          />
        </div>

        {/* Verification Context Input */}
        <div className="space-y-2 p-4 border border-border border-1 border-blue-300 dark:border-blue-700 rounded-md bg-blue-50 dark:bg-blue-950/20">
          <Label htmlFor="verification-context" className="text-blue-700 dark:text-blue-400 font-medium">
            Verification Context (Base85)
          </Label>
          <Textarea
            id="verification-context"
            placeholder="Paste the verification context here"
            value={verifyContext}
            onChange={(e) => setVerifyContext(e.target.value)}
            className="font-mono text-xs bg-white dark:bg-gray-950 border-blue-200 dark:border-blue-800/50 focus-visible:ring-blue-500 rounded-md"
            rows={3}
          />
          <p className="text-xs text-muted-foreground">
            Contains all parameters needed for verification (circuit, k, strategy)
          </p>
        </div>

        {/* Public Signals Table */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label>Public Signals</Label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={addSignalRow}
              className="rounded-md"
            >
              <Plus className="h-4 w-4 mr-1" />
              Add Signal
            </Button>
          </div>
          <div className="border rounded-md overflow-hidden border-1 border-gray-300 dark:border-gray-600">
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
                {signalRows.filter(r => !r.isOutput).map((row) => {
                  const originalIndex = signalRows.indexOf(row);
                  return (
                    <tr key={originalIndex} className="border-b border-border last:border-b-0 hover:bg-muted/30">
                      <td className="px-4 py-2">
                        <span className="inline-flex items-center px-2 py-1 rounded-md bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium">
                          Public
                        </span>
                      </td>
                      <td className="px-4 py-2">
                        <Input
                          value={row.name}
                          onChange={(e) => updateSignalRow(originalIndex, 'name', e.target.value)}
                          placeholder="Signal name"
                          className="h-8 text-sm border border-gray-300 dark:border-gray-600 bg-white/50 dark:bg-gray-950/50 focus-visible:ring-1 rounded-md"
                        />
                      </td>
                      <td className="px-4 py-2">
                        <Input
                          value={row.value}
                          onChange={(e) => updateSignalRow(originalIndex, 'value', e.target.value)}
                          placeholder="Value"
                          className="h-8 font-mono text-sm border border-gray-300 dark:border-gray-600 bg-white/50 dark:bg-gray-950/50 focus-visible:ring-1 rounded-md"
                        />
                      </td>
                      <td className="px-4 py-2">
                        <Select value={row.encoding || 'decimal'} onValueChange={(value) => updateSignalRow(originalIndex, 'encoding', value as 'decimal' | 'hex' | 'base58' | 'base64' | 'base85')}>
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
                          onClick={() => removeSignalRow(originalIndex)}
                          disabled={signalRows.length === 1}
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
        {signalRows.find(r => r.isOutput) && (
          <div className="space-y-2">
            <Label>Output Signal</Label>
            <div className="border border-green-300 dark:border-green-700 rounded-md bg-green-50/30 dark:bg-green-950/10 p-4">
              {(() => {
                const outputRow = signalRows.find(r => r.isOutput);
                const outputIndex = signalRows.indexOf(outputRow!);
                return (
                  <div className="flex items-center gap-4">
                    <span className="inline-flex items-center px-2 py-1 rounded-md bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 text-xs font-medium flex-shrink-0">
                      Output
                    </span>
                    <Input
                      value={outputRow!.name}
                      onChange={(e) => updateSignalRow(outputIndex, 'name', e.target.value)}
                      placeholder="Output signal name"
                      className="h-8 text-sm border border-green-300 dark:border-green-700 bg-white dark:bg-gray-950 rounded-md flex-1"
                    />
                    <Input
                      value={outputRow!.value}
                      onChange={(e) => updateSignalRow(outputIndex, 'value', e.target.value)}
                      placeholder="Result value"
                      className="h-8 font-mono text-sm border border-green-300 dark:border-green-700 bg-white dark:bg-gray-950 rounded-md flex-1"
                    />
                    <Select
                      value={outputRow!.encoding || 'decimal'}
                      onValueChange={(value) => updateSignalRow(outputIndex, 'encoding', value as 'decimal' | 'hex' | 'base58' | 'base64' | 'base85')}
                    >
                      <SelectTrigger className="h-8 text-sm w-32 border border-green-300 dark:border-green-700 bg-white dark:bg-gray-950 rounded-md">
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
                  </div>
                );
              })()}
            </div>
          </div>
        )}

        {/* Verify Button */}
        <div className="flex justify-center">
          <Button
            onClick={handleVerify}
            disabled={verifying}
            className="min-w-48 rounded-md"
            size="lg"
          >
            {verifying ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Verifying Proof...
              </>
            ) : (
              'Verify Proof'
            )}
          </Button>
        </div>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive" className="rounded-md">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Result Display */}
        {result && (
          <Alert variant={result.valid ? 'default' : 'destructive'} className="rounded-md">
            {result.valid ? (
              <CheckCircle2 className="h-4 w-4 !text-green-600 dark:!text-green-400" />
            ) : (
              <XCircle className="h-4 w-4" />
            )}
            <AlertTitle className={result.valid ? 'text-green-600 dark:text-green-400' : ''}>
              {result.valid ? 'Proof is Valid!' : 'Proof is Invalid'}
            </AlertTitle>
            <AlertDescription>
              {result.valid
                ? 'The zero-knowledge proof has been successfully verified.'
                : result.error || 'The proof verification failed.'}
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
}