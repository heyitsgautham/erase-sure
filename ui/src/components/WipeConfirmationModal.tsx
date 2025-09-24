import { useState } from 'react';
import { Device } from '../contexts/AppContext';

interface WipeConfirmationModalProps {
    device: Device;
    policy: string;
    onConfirm: (userInput: string) => void;
    onCancel: () => void;
    isOpen: boolean;
}

function WipeConfirmationModal({ device, policy, onConfirm, onCancel, isOpen }: WipeConfirmationModalProps) {
    const [userInput, setUserInput] = useState('');
    const expectedInput = `WIPE ${device.serial || 'UNKNOWN'}`;

    const handleConfirm = () => {
        onConfirm(userInput);
        setUserInput(''); // Clear input after confirm
    };

    if (!isOpen) return null;

    const isValidInput = userInput === expectedInput;

    return (
        <div className="modal-overlay" style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0,0,0,0.7)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000
        }}>
            <div className="modal-content" style={{
                backgroundColor: 'white',
                borderRadius: '8px',
                padding: '1.5rem',
                maxWidth: '600px',
                width: '95%',
                maxHeight: '95vh',
                overflow: 'auto',
                wordWrap: 'break-word',
                overflowWrap: 'break-word'
            }}>
                <div style={{ textAlign: 'center', marginBottom: '2rem' }}>
                    <div style={{ fontSize: '4rem', marginBottom: '1rem' }}>⚠️</div>
                    <h2 style={{ color: '#dc2626', fontWeight: 'bold', fontSize: '1.5rem', marginBottom: '1rem' }}>
                        DESTRUCTIVE WIPE CONFIRMATION
                    </h2>
                    <div style={{ 
                        backgroundColor: '#fef2f2', 
                        border: '1px solid #fecaca', 
                        borderRadius: '6px', 
                        padding: '1rem',
                        textAlign: 'left',
                        marginBottom: '1.5rem'
                    }}>
                        <h3 style={{ color: '#dc2626', fontWeight: 'bold', marginBottom: '0.5rem' }}>
                            THIS WILL PERMANENTLY DESTROY ALL DATA
                        </h3>
                        <p style={{ color: '#991b1b', marginBottom: '0.5rem' }}>
                            This operation cannot be undone. All data on the selected device will be 
                            irreversibly destroyed using NIST-aligned secure wiping procedures.
                        </p>
                    </div>
                </div>

                <div style={{ marginBottom: '1.5rem', textAlign: 'left' }}>
                    <h3 style={{ fontWeight: 'bold', marginBottom: '1rem' }}>Device Information:</h3>
                    <div style={{ 
                        backgroundColor: '#f8fafc', 
                        border: '1px solid #e2e8f0', 
                        borderRadius: '6px', 
                        padding: '1rem' 
                    }}>
                        <div style={{ 
                            display: 'grid', 
                            gridTemplateColumns: 'auto 1fr', 
                            gap: '0.5rem 1rem', 
                            fontSize: '0.9rem',
                            overflow: 'hidden'
                        }}>
                            <div><strong>Model:</strong></div>
                            <div>{device.model}</div>
                            <div><strong>Serial:</strong></div>
                            <div style={{ 
                                fontFamily: 'monospace',
                                wordBreak: 'break-all',
                                fontSize: '0.8rem',
                                overflow: 'hidden',
                                textOverflow: 'ellipsis',
                                lineHeight: '1.4'
                            }}>
                                {device.serial || 'Unknown'}
                            </div>
                            <div><strong>Capacity:</strong></div>
                            <div>{(device.capacity / (1024 ** 3)).toFixed(1)} GB</div>
                            <div><strong>Path:</strong></div>
                            <div style={{ 
                                fontFamily: 'monospace',
                                wordBreak: 'break-all'
                            }}>{device.path}</div>
                            <div><strong>Wipe Policy:</strong></div>
                            <div>
                                <span style={{ 
                                    backgroundColor: '#dbeafe', 
                                    color: '#1e40af', 
                                    padding: '0.25rem 0.5rem', 
                                    borderRadius: '4px', 
                                    fontSize: '0.8rem' 
                                }}>
                                    {policy}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                <div style={{ marginBottom: '1.5rem' }}>
                    <label style={{ 
                        display: 'block', 
                        fontWeight: 'bold', 
                        marginBottom: '0.5rem',
                        color: '#374151'
                    }}>
                        To proceed, type exactly:
                    </label>
                    <div style={{
                        backgroundColor: '#f3f4f6', 
                        padding: '0.5rem', 
                        borderRadius: '4px',
                        fontFamily: 'monospace',
                        fontSize: '0.85rem',
                        wordBreak: 'break-all',
                        marginBottom: '0.5rem',
                        border: '1px solid #d1d5db'
                    }}>
                        {expectedInput}
                    </div>
                    <input
                        type="text"
                        value={userInput}
                        onChange={(e) => setUserInput(e.target.value)}
                        placeholder="Type the confirmation text above..."
                        style={{
                            width: '100%',
                            padding: '0.75rem',
                            border: '2px solid #e5e7eb',
                            borderRadius: '6px',
                            fontSize: '0.9rem',
                            fontFamily: 'monospace',
                            backgroundColor: isValidInput ? '#f0f9ff' : '#ffffff',
                            borderColor: isValidInput ? '#0ea5e9' : '#e5e7eb',
                            wordBreak: 'break-all'
                        }}
                        autoFocus
                    />
                    {userInput && !isValidInput && (
                        <div style={{ 
                            color: '#dc2626', 
                            fontSize: '0.8rem', 
                            marginTop: '0.5rem',
                            wordBreak: 'break-all'
                        }}>
                            Input does not match the required confirmation text shown above.
                        </div>
                    )}
                </div>

                <div style={{ 
                    backgroundColor: '#fffbeb', 
                    border: '1px solid #fde68a', 
                    borderRadius: '6px', 
                    padding: '1rem',
                    marginBottom: '1.5rem'
                }}>
                    <h4 style={{ color: '#92400e', fontWeight: 'bold', marginBottom: '0.5rem' }}>
                        Safety Requirements Met:
                    </h4>
                    <ul style={{ color: '#78350f', fontSize: '0.9rem', paddingLeft: '1.5rem' }}>
                        <li>SECUREWIPE_DANGER=1 environment variable is set</li>
                        <li>Device is not a critical system disk</li>
                        <li>Explicit user confirmation with device serial required</li>
                        <li>Operation will be logged and certificated</li>
                    </ul>
                </div>

                <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
                    <button
                        onClick={onCancel}
                        style={{
                            padding: '0.75rem 1.5rem',
                            backgroundColor: '#6b7280',
                            color: 'white',
                            border: 'none',
                            borderRadius: '6px',
                            fontWeight: 'bold',
                            cursor: 'pointer'
                        }}
                    >
                        Cancel
                    </button>
                    <button
                        onClick={handleConfirm}
                        disabled={!isValidInput}
                        style={{
                            padding: '0.75rem 1.5rem',
                            backgroundColor: isValidInput ? '#dc2626' : '#9ca3af',
                            color: 'white',
                            border: 'none',
                            borderRadius: '6px',
                            fontWeight: 'bold',
                            cursor: isValidInput ? 'pointer' : 'not-allowed'
                        }}
                    >
                        🗑️ EXECUTE WIPE
                    </button>
                </div>
            </div>
        </div>
    );
}

export default WipeConfirmationModal;