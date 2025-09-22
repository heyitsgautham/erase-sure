import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe } from '../hooks/useSecureWipe';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import LogViewer from '../components/LogViewer';
import WipeConfirmationModal from '../components/WipeConfirmationModal';

interface WipeProgress {
    session_id: string;
    device: string;
    policy: string;
    timestamp: string;
    status: 'starting' | 'in_progress' | 'completed' | 'failed';
    current_step?: string;
    certificate_id?: string;
}

function DestructiveWipe() {
    const navigate = useNavigate();
    const { state, addToast } = useApp();
    const { logs, clearLogs } = useSecureWipe();
    const [showConfirmation, setShowConfirmation] = useState(false);
    const [selectedPolicy, setSelectedPolicy] = useState<'clear' | 'purge'>('purge');
    const [wipeProgress, setWipeProgress] = useState<WipeProgress | null>(null);
    const [backupCertId, setBackupCertId] = useState<string>('');

    useEffect(() => {
        // Listen for wipe progress events
        const unlisten = listen('wipe://start', (event: any) => {
            const progress = event.payload as WipeProgress;
            setWipeProgress({ ...progress, status: 'starting' });
            addToast(`Starting wipe operation on ${progress.device}`, 'info');
        });

        return () => {
            unlisten.then(fn => fn());
        };
    }, []);

    const handleStartWipe = () => {
        if (!state.selectedDevice) {
            addToast('Please select a device first', 'error');
            return;
        }

        if (state.selectedDevice.risk_level === 'CRITICAL') {
            addToast('Cannot wipe CRITICAL devices unless running from ISO mode', 'error');
            return;
        }

        setShowConfirmation(true);
    };

    const handleConfirmWipe = async (userInput: string) => {
        if (!state.selectedDevice) return;

        try {
            const confirmation = {
                device: state.selectedDevice.path,
                serial: state.selectedDevice.serial || 'UNKNOWN',
                policy: selectedPolicy,
                user_input: userInput
            };

            clearLogs();
            setShowConfirmation(false);
            setWipeProgress(prev => prev ? { ...prev, status: 'in_progress' } : null);
            
            addToast('Executing destructive wipe...', 'info');
            
            await invoke('execute_destructive_wipe', {
                confirmation,
                backupCertId: backupCertId || null
            });
            
            setWipeProgress(prev => prev ? { ...prev, status: 'completed' } : null);
            addToast('Wipe operation completed successfully!', 'success');
            
        } catch (error) {
            console.error('Wipe failed:', error);
            setWipeProgress(prev => prev ? { ...prev, status: 'failed' } : null);
            addToast(`Wipe operation failed: ${error}`, 'error');
        }
    };

    const handleCancelConfirmation = () => {
        setShowConfirmation(false);
    };

    const handleBackToDiscovery = () => {
        navigate('/discover');
    };

    if (!state.selectedDevice) {
        return (
            <div style={{ maxWidth: '800px', margin: '0 auto' }}>
                <div className="card text-center" style={{ padding: '3rem' }}>
                    <div style={{ fontSize: '4rem', marginBottom: '1rem', opacity: 0.3 }}>🗑️</div>
                    <h3 className="font-semibold mb-2">No Device Selected</h3>
                    <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                        Please discover and select a device before performing destructive wipe.
                    </p>
                    <button
                        className="btn btn-primary"
                        onClick={handleBackToDiscovery}
                    >
                        🔍 Go to Device Discovery
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
            <div className="mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    🗑️ Destructive Wipe Operations
                </h2>
                <div className="alert alert-warning mb-6">
                    <h4 className="font-semibold mb-2">⚠️ WARNING: DESTRUCTIVE OPERATION</h4>
                    <p className="text-sm">
                        This mode performs real, irreversible data destruction. All data on the selected device 
                        will be permanently wiped using NIST-aligned secure deletion procedures.
                    </p>
                </div>
            </div>

            {/* Device Information */}
            <div className="card mb-6">
                <h3 className="font-semibold mb-4">Selected Device</h3>
                <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                        <span className="font-medium">Model:</span>
                        <div>{state.selectedDevice.model}</div>
                    </div>
                    <div>
                        <span className="font-medium">Serial:</span>
                        <div style={{ fontFamily: 'monospace' }}>{state.selectedDevice.serial || 'Unknown'}</div>
                    </div>
                    <div>
                        <span className="font-medium">Capacity:</span>
                        <div>{(state.selectedDevice.capacity / (1024 ** 3)).toFixed(1)} GB</div>
                    </div>
                    <div>
                        <span className="font-medium">Risk Level:</span>
                        <div>
                            <span className={`risk-badge ${state.selectedDevice.risk_level.toLowerCase()}`}>
                                {state.selectedDevice.risk_level}
                            </span>
                        </div>
                    </div>
                    <div>
                        <span className="font-medium">Path:</span>
                        <div style={{ fontFamily: 'monospace' }}>{state.selectedDevice.path}</div>
                    </div>
                    <div>
                        <span className="font-medium">Bus Type:</span>
                        <div>{state.selectedDevice.bus || 'Unknown'}</div>
                    </div>
                </div>
            </div>

            {/* Wipe Configuration */}
            <div className="card mb-6">
                <h3 className="font-semibold mb-4">Wipe Configuration</h3>
                
                <div className="mb-4">
                    <label className="font-medium mb-2 block">Wipe Policy:</label>
                    <div className="space-y-2">
                        <label className="flex items-center gap-2">
                            <input
                                type="radio"
                                value="clear"
                                checked={selectedPolicy === 'clear'}
                                onChange={(e) => setSelectedPolicy(e.target.value as any)}
                            />
                            <span><strong>CLEAR</strong> - Single zero overwrite (fastest)</span>
                        </label>
                        <label className="flex items-center gap-2">
                            <input
                                type="radio"
                                value="purge"
                                checked={selectedPolicy === 'purge'}
                                onChange={(e) => setSelectedPolicy(e.target.value as any)}
                            />
                            <span><strong>PURGE</strong> - Random overwrite + verification (recommended)</span>
                        </label>
                    </div>
                </div>

                {/* DESTROY Information */}
                <div className="card mb-6">
                    <h3 className="font-semibold mb-4">About DESTROY Level</h3>
                    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <div className="flex items-start gap-3">
                            <div className="text-2xl">ℹ️</div>
                            <div>
                                <h4 className="font-semibold text-blue-900 mb-2">Physical Destruction (DESTROY)</h4>
                                <p className="text-blue-800 text-sm mb-2">
                                    DESTROY refers to physical destruction of the storage media according to NIST SP 800-88 guidelines.
                                    This is not a software operation and cannot be performed by this application.
                                </p>
                                <p className="text-blue-800 text-sm">
                                    For DESTROY level assurance, physically destroy the device using methods such as:
                                    shredding, incineration, or degaussing (for magnetic media).
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="mb-4">
                    <label className="font-medium mb-2 block">Link to Backup Certificate (Optional):</label>
                    <input
                        type="text"
                        value={backupCertId}
                        onChange={(e) => setBackupCertId(e.target.value)}
                        placeholder="Enter backup certificate ID to link this wipe"
                        style={{
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            fontFamily: 'monospace'
                        }}
                    />
                    <div style={{ fontSize: '0.8rem', color: '#6b7280', marginTop: '0.5rem' }}>
                        If provided, this wipe will be linked to the backup certificate for chain of custody.
                    </div>
                </div>
            </div>

            {/* Wipe Progress */}
            {wipeProgress && (
                <div className="card mb-6">
                    <h3 className="font-semibold mb-4">Wipe Progress</h3>
                    <div className="mb-4">
                        <div className="flex items-center gap-2 mb-2">
                            {wipeProgress.status === 'in_progress' && (
                                <div style={{
                                    width: '16px',
                                    height: '16px',
                                    border: '2px solid #3b82f6',
                                    borderTopColor: 'transparent',
                                    borderRadius: '50%',
                                    animation: 'spin 1s linear infinite'
                                }}></div>
                            )}
                            <span className="font-medium">
                                Status: {wipeProgress.status.replace('_', ' ').toUpperCase()}
                            </span>
                        </div>
                        <div className="text-sm text-gray-600">
                            Session: {wipeProgress.session_id}
                        </div>
                        <div className="text-sm text-gray-600">
                            Started: {new Date(wipeProgress.timestamp).toLocaleString()}
                        </div>
                    </div>
                </div>
            )}

            {/* Action Buttons */}
            <div className="card mb-6">
                <div className="flex gap-4">
                    <button
                        className="btn btn-secondary"
                        onClick={handleBackToDiscovery}
                    >
                        ← Back to Discovery
                    </button>
                    <button
                        className="btn btn-danger"
                        onClick={handleStartWipe}
                        disabled={!!wipeProgress && wipeProgress.status === 'in_progress'}
                        style={{
                            backgroundColor: '#dc2626',
                            color: 'white',
                            fontWeight: 'bold'
                        }}
                    >
                        🗑️ Start Destructive Wipe
                    </button>
                </div>
            </div>

            {/* Log Output */}
            {logs.length > 0 && (
                <div className="card">
                    <h3 className="font-semibold mb-4">Operation Logs</h3>
                    <LogViewer title="Wipe Operation Logs" />
                </div>
            )}

            {/* Confirmation Modal */}
            <WipeConfirmationModal
                device={state.selectedDevice}
                policy={selectedPolicy.toUpperCase()}
                onConfirm={handleConfirmWipe}
                onCancel={handleCancelConfirmation}
                isOpen={showConfirmation}
            />
        </div>
    );
}

export default DestructiveWipe;