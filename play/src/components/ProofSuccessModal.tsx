import { Button } from './ui/button';
import { CheckCircle, Download } from 'lucide-react';
import { type ProveResponse } from '../hooks/useZKPlex';

interface ProofSuccessModalProps {
  result: ProveResponse;
  provingTime: number;
  circuit: string;
  onClose: () => void;
  onDownload: () => void;
  onNavigateToVerify: () => void;
}

export function ProofSuccessModal({
  result,
  provingTime,
  onClose,
  onDownload,
  onNavigateToVerify,
}: ProofSuccessModalProps) {
  return (
    <div
      className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-8"
      onClick={onClose}
    >
      <div
        className="bg-white dark:bg-gray-950 rounded-lg shadow-2xl p-8 max-w-4xl w-full"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex flex-col items-center gap-6">
          {/* Success Icon */}
          <CheckCircle className="h-16 w-16 text-green-500" />

          {/* Title */}
          <div className="text-center">
            <h2 className="text-2xl font-bold mb-2">Proof Generated Successfully!</h2>
            <p className="text-muted-foreground">
              Your zero-knowledge proof has been generated and is ready to verify
            </p>
          </div>

          {/* Statistics Grid */}
          <div className="grid grid-cols-4 gap-4 w-full">
            <div className="p-4 border rounded-md bg-muted/30 text-center">
              <div className="text-xl font-semibold">
                {(() => {
                  const bytes = new Blob([result.proof]).size;
                  return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(2)} KB`;
                })()}
              </div>
              <p className="text-xs text-muted-foreground mt-1">Proof Size</p>
            </div>
            <div className="p-4 border rounded-md bg-muted/30 text-center">
              <div className="text-xl font-semibold">{(provingTime / 1000).toFixed(2)}s</div>
              <p className="text-xs text-muted-foreground mt-1">Proving Time</p>
            </div>
            <div className="p-4 border rounded-md bg-muted/30 text-center">
              <div className="text-xl font-semibold">k={result.debug?.k}</div>
              <p className="text-xs text-muted-foreground mt-1">Circuit Size</p>
            </div>
            <div className="p-4 border rounded-md bg-green-50 dark:bg-green-950/20 border-green-300 dark:border-green-700 text-center">
              <div className="text-xl font-semibold text-green-700 dark:text-green-300">
                {(() => {
                  const outputSignalName = result.debug?.output_signal;
                  if (!outputSignalName) return 'N/A';
                  const outputValue = result.public_signals[outputSignalName]?.value;
                  return outputValue || 'N/A';
                })()}
              </div>
              <p className="text-xs text-green-600 dark:text-green-400 mt-1">Output</p>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-4 w-full">
            <Button
              onClick={onDownload}
              variant="outline"
              className="flex-1"
              size="lg"
            >
              <Download className="mr-2 h-4 w-4" />
              Download Proof
            </Button>
            <Button
              onClick={onNavigateToVerify}
              className="flex-1"
              size="lg"
            >
              <CheckCircle className="mr-2 h-4 w-4" />
              Verify Proof
            </Button>
          </div>

          {/* Close Button */}
          <Button
            onClick={onClose}
            variant="ghost"
            className="mt-2"
          >
            Close
          </Button>
        </div>
      </div>
    </div>
  );
}