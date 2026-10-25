import type { VerifyResult } from '../../types/securewipe';

interface VerifyResultModalProps {
    result: VerifyResult;
    onClose: () => void;
}

/** The portal's verdict on one certificate, check by check. */
function VerifyResultModal({ result, onClose }: VerifyResultModalProps) {
    return (
        <div className="modal-overlay" style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000
        }}>
            <div className="modal-content card" style={{
                maxWidth: '500px',
                width: '90%',
                maxHeight: '80vh',
                overflow: 'auto',
                padding: '2rem'
            }}>
                <div style={{ textAlign: 'center', marginBottom: '1.5rem' }}>
                    <h3 className="font-semibold mb-2">Certificate Verification Result</h3>
                    <p style={{ fontSize: '0.9rem', color: '#64748b' }}>
                        Certificate ID: {result.cert_summary?.cert_id}
                    </p>
                </div>

                <div className="space-y-3 mb-6">
                    <div className="flex justify-between items-center">
                        <span>Schema Valid:</span>
                        <span className={`font-semibold ${result.schema_valid ? 'text-green-600' : 'text-red-600'}`}>
                            {result.schema_valid ? '✓ Yes' : '✗ No'}
                        </span>
                    </div>

                    <div className="flex justify-between items-center">
                        <span>Signature Valid:</span>
                        <span className={`font-semibold ${
                            result.signature_valid === null ? 'text-gray-500' :
                            result.signature_valid ? 'text-green-600' : 'text-red-600'
                        }`}>
                            {result.signature_valid === null ? '- N/A' :
                             result.signature_valid ? '✓ Yes' : '✗ No'}
                        </span>
                    </div>

                    <div className="flex justify-between items-center">
                        <span>Hash Valid:</span>
                        <span className={`font-semibold ${
                            result.hash_valid === null ? 'text-gray-500' :
                            result.hash_valid ? 'text-green-600' : 'text-red-600'
                        }`}>
                            {result.hash_valid === null ? '- N/A' :
                             result.hash_valid ? '✓ Yes' : '✗ No'}
                        </span>
                    </div>

                    {result.chain_valid !== null && (
                        <div className="flex justify-between items-center">
                            <span>Chain Valid:</span>
                            <span className={`font-semibold ${result.chain_valid ? 'text-green-600' : 'text-red-600'}`}>
                                {result.chain_valid ? '✓ Yes' : '✗ No'}
                            </span>
                        </div>
                    )}
                </div>

                {result.errors.length > 0 && (
                    <div className="mb-6">
                        <h4 className="font-semibold mb-2 text-red-600">Issues Found:</h4>
                        <ul className="text-sm space-y-1">
                            {result.errors.map((error: string, index: number) => (
                                <li key={index} className="text-red-600">• {error}</li>
                            ))}
                        </ul>
                    </div>
                )}

                <div className="flex gap-3 justify-end">
                    <button
                        className="btn btn-secondary"
                        onClick={onClose}
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
}

export default VerifyResultModal;
