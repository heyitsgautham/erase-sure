import type { VerifyResult } from '../../types/securewipe';

interface CheckRowProps {
    label: string;
    value: boolean | null;
    /** Show a null check as not applicable rather than as a failure. */
    showNotApplicable?: boolean;
}

function CheckRow({ label, value, showNotApplicable = false }: CheckRowProps) {
    const notApplicable = showNotApplicable && value === null;
    const tone = notApplicable ? 'text-gray-500' : value ? 'text-green-600' : 'text-red-600';
    return (
        <div className="flex justify-between items-center">
            <span>{label}:</span>
            <span className={`font-semibold ${tone}`}>{notApplicable ? '- N/A' : value ? '✓ Yes' : '✗ No'}</span>
        </div>
    );
}

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
                    <CheckRow label="Schema Valid" value={result.schema_valid} />
                    <CheckRow label="Signature Valid" value={result.signature_valid} showNotApplicable />
                    <CheckRow label="Hash Valid" value={result.hash_valid} showNotApplicable />
                    {result.chain_valid !== null && <CheckRow label="Chain Valid" value={result.chain_valid} />}
                </div>

                {result.errors.length > 0 && (
                    <div className="mb-6">
                        <h4 className="font-semibold mb-2 text-red-600">Issues Found:</h4>
                        <ul className="text-sm space-y-1" style={{ listStyle: 'none', paddingLeft: 0 }}>
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
