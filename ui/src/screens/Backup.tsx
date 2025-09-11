import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe } from '../hooks/useSecureWipe';
import LogViewer from '../components/LogViewer';
import FileLink from '../components/FileLink';

function Backup() {
    const navigate = useNavigate();
    const { state, addToast } = useApp();
    const { runBackup } = useSecureWipe();
    const [destination, setDestination] = useState('~/SecureWipe/backups');
    const [customPaths, setCustomPaths] = useState('');
    const [signKeyPath, setSignKeyPath] = useState('');

    const handleSelectDestination = async () => {
        try {
            // In production, use Tauri's dialog API
            // const selected = await open({ directory: true });
            // if (selected) setDestination(selected);
            addToast('Folder selection will be implemented with Tauri dialog API', 'info');
        } catch (error) {
            console.error('Failed to select destination:', error);
        }
    };

    const handleRunBackup = async () => {
        if (!state.selectedDevice) {
            addToast('Please select a device first', 'error');
            return;
        }

        if (!destination.trim()) {
            addToast('Please specify a backup destination', 'error');
            return;
        }

        try {
            addToast(`Starting backup of ${state.selectedDevice.model}...`, 'info');
            await runBackup(state.selectedDevice.path, destination, signKeyPath || undefined);

            addToast('Backup completed successfully! 🎉', 'success');

            // Navigate to certificates after successful backup
            setTimeout(() => {
                navigate('/certificates');
            }, 2000);
        } catch (error) {
            console.error('Backup failed:', error);
            addToast('Backup operation failed. Please check logs for details.', 'error');
        }
    };

    const handleBackToDiscover = () => {
        navigate('/discover');
    };

    const handleViewCertificates = () => {
        navigate('/certificates');
    };

    if (!state.selectedDevice) {
        return (
            <div style={{ maxWidth: '800px', margin: '0 auto' }}>
                <div className="card text-center" style={{ padding: '3rem' }}>
                    <div style={{ fontSize: '4rem', marginBottom: '1rem', opacity: 0.3 }}>📦</div>
                    <h3 className="font-semibold mb-2">No Device Selected</h3>
                    <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                        Please discover and select a device before running backup operations.
                    </p>
                    <button
                        className="btn btn-primary"
                        onClick={handleBackToDiscover}
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
                    Encrypted Backup
                </h2>
                <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                    Create an encrypted backup of {state.selectedDevice.model} before wiping.
                    All data is protected with AES-256-CTR encryption and integrity verification.
                </p>

                {/* Device Info */}
                <div className="card mb-6">
                    <h3 className="font-semibold mb-4">Source Device</h3>
                    <div className="grid grid-cols-3 gap-4 text-sm">
                        <div>
                            <span className="font-medium">Model:</span>
                            <div>{state.selectedDevice.model}</div>
                        </div>
                        <div>
                            <span className="font-medium">Capacity:</span>
                            <div>{(state.selectedDevice.capacity / (1024 ** 3)).toFixed(1)} GB</div>
                        </div>
                        <div>
                            <span className="font-medium">Serial:</span>
                            <div>{state.selectedDevice.serial}</div>
                        </div>
                    </div>
                </div>

                {/* Backup Configuration */}
                <div className="card mb-6">
                    <h3 className="font-semibold mb-4">Backup Configuration</h3>

                    <div className="form-group">
                        <label className="form-label">Destination Folder</label>
                        <div className="flex gap-2">
                            <input
                                type="text"
                                className="form-input"
                                value={destination}
                                onChange={(e) => setDestination(e.target.value)}
                                placeholder="~/SecureWipe/backups"
                            />
                            <button
                                className="btn btn-secondary"
                                onClick={handleSelectDestination}
                            >
                                📁 Browse
                            </button>
                        </div>
                    </div>

                    <div className="form-group">
                        <label className="form-label">Custom Include Paths (Optional)</label>
                        <textarea
                            className="form-input"
                            value={customPaths}
                            onChange={(e) => setCustomPaths(e.target.value)}
                            placeholder="Additional files/folders to include, one per line"
                            rows={3}
                            style={{ resize: 'vertical' }}
                        />
                        <div className="text-xs" style={{ color: '#64748b', marginTop: '0.5rem' }}>
                            Leave empty to backup entire device. Specify custom paths for selective backup.
                        </div>
                    </div>

                    <div className="form-group">
                        <label className="form-label">Signing Key Path (Optional)</label>
                        <input
                            type="text"
                            className="form-input"
                            value={signKeyPath}
                            onChange={(e) => setSignKeyPath(e.target.value)}
                            placeholder="Path to private key for certificate signing"
                        />
                        <div className="text-xs" style={{ color: '#64748b', marginTop: '0.5rem' }}>
                            Leave empty to use default development key. Specify custom key for production use.
                        </div>
                    </div>

                    {/* Encryption Info */}
                    <div className="alert alert-info">
                        <h4 className="font-semibold mb-2">🔐 Encryption Details</h4>
                        <div className="text-sm grid grid-cols-2 gap-4">
                            <div>
                                <span className="font-medium">Algorithm:</span> AES-256-CTR
                            </div>
                            <div>
                                <span className="font-medium">Key Derivation:</span> PBKDF2
                            </div>
                            <div>
                                <span className="font-medium">Integrity:</span> HMAC-SHA256
                            </div>
                            <div>
                                <span className="font-medium">Verification:</span> N=5 sample checks
                            </div>
                        </div>
                    </div>
                </div>

                {/* Action Button */}
                <button
                    className="btn btn-primary btn-large mb-6"
                    onClick={handleRunBackup}
                    disabled={state.isLoading}
                    style={{ width: '100%' }}
                >
                    {state.isLoading ? '🔄 Running Backup...' : '🛡️ Run Backup (Encrypted)'}
                </button>
            </div>

            {/* Operation Logs */}
            {state.logs.length > 0 && (
                <div className="mb-6">
                    <LogViewer
                        logs={state.logs}
                        title="Backup Progress"
                    />
                </div>
            )}

            {/* Backup Results */}
            {state.backupResult && (
                <div className="mb-6">
                    <div className="card">
                        <h3 className="font-semibold mb-4">✅ Backup Completed Successfully</h3>

                        <div className="grid grid-cols-2 gap-6 mb-6">
                            <div>
                                <h4 className="font-semibold mb-3">Backup Summary</h4>
                                <div className="text-sm space-y-2">
                                    <div>
                                        <span className="font-medium">Location:</span>
                                        <div className="mt-1">{state.backupResult.backup_path}</div>
                                    </div>
                                    <div>
                                        <span className="font-medium">Integrity Checks:</span>
                                        <div className="mt-1">{state.backupResult.integrity_checks} samples verified ✓</div>
                                    </div>
                                </div>
                            </div>

                            <div>
                                <h4 className="font-semibold mb-3">Generated Files</h4>
                                <div className="space-y-2">
                                    <FileLink
                                        path={state.backupResult.backup_path}
                                        label="Open Backup Folder"
                                        type="folder"
                                    />
                                    <FileLink
                                        path={state.backupResult.manifest_path}
                                        label="Backup Manifest"
                                        type="json"
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4 mb-6">
                            {state.backupResult.certificate_json_path && (
                                <FileLink
                                    path={state.backupResult.certificate_json_path}
                                    label="Backup Certificate (JSON)"
                                    type="json"
                                />
                            )}
                            {state.backupResult.certificate_pdf_path && (
                                <FileLink
                                    path={state.backupResult.certificate_pdf_path}
                                    label="Backup Certificate (PDF)"
                                    type="pdf"
                                />
                            )}
                        </div>

                        <div className="flex gap-4">
                            <button
                                className="btn btn-primary"
                                onClick={handleViewCertificates}
                            >
                                📜 View All Certificates
                            </button>

                            <button
                                className="btn btn-secondary"
                                onClick={() => window.open('http://localhost:8000', '_blank')}
                            >
                                🌐 Open in Portal Verify
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Current Operation Status */}
            {state.currentOperation && (
                <div className="alert alert-info">
                    <div className="flex items-center gap-2">
                        <div style={{
                            width: '20px',
                            height: '20px',
                            border: '2px solid #3b82f6',
                            borderTopColor: 'transparent',
                            borderRadius: '50%',
                            animation: 'spin 1s linear infinite'
                        }}></div>
                        <span>{state.currentOperation}</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default Backup;
