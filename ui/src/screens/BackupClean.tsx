import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { open } from '@tauri-apps/api/dialog';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe } from '../hooks/useSecureWipe';
import LogViewer from '../components/LogViewer';
import FileLink from '../components/FileLink';
import Progress from '../components/Progress';
import FileBrowser from '../components/FileBrowser';

function Backup() {
    const navigate = useNavigate();
    const { state, addToast, dispatch } = useApp();
    const { backup, running } = useSecureWipe();
    const [destination, setDestination] = useState('~/SecureWipe/backups');
    const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
    const [signKeyPath, setSignKeyPath] = useState('');
    const [useDefaultPaths, setUseDefaultPaths] = useState(true);
    const [showFileBrowser, setShowFileBrowser] = useState(false);

    const handleSelectDestination = async () => {
        try {
            // Use Tauri's dialog API to open a folder selection dialog
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Select Backup Destination Folder',
                defaultPath: destination.startsWith('~') ? undefined : destination
            });
            
            if (selected && typeof selected === 'string') {
                setDestination(selected);
                addToast('Backup destination updated', 'success');
            }
        } catch (error) {
            console.error('Failed to select destination:', error);
            addToast('Failed to open folder selection dialog', 'error');
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

            // Mock progress tracking for demonstration
            const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

            // Step 1: Initialize
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Encrypted Backup in Progress',
                    currentStep: 1,
                    totalSteps: 5,
                    currentStepName: 'Initializing backup process...',
                    percentage: 0
                }
            });
            await delay(800);

            // Step 2: Encryption Setup
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Encrypted Backup in Progress',
                    currentStep: 2,
                    totalSteps: 5,
                    currentStepName: 'Setting up AES-256-CTR encryption...',
                    percentage: 25
                }
            });
            await delay(1000);

            // Step 3: Data Copy
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Encrypted Backup in Progress',
                    currentStep: 3,
                    totalSteps: 5,
                    currentStepName: 'Copying and encrypting data...',
                    percentage: 50
                }
            });
            await delay(1200);

            // Step 4: Verification
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Encrypted Backup in Progress',
                    currentStep: 4,
                    totalSteps: 5,
                    currentStepName: 'Verifying backup integrity...',
                    percentage: 80
                }
            });
            await delay(900);

            // Step 5: Certificate Generation
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Encrypted Backup in Progress',
                    currentStep: 5,
                    totalSteps: 5,
                    currentStepName: 'Generating signed certificates...',
                    percentage: 100
                }
            });
            await delay(800);

            const includePaths = !useDefaultPaths && selectedFiles.length > 0 
                ? selectedFiles 
                : undefined;

            await backup({
                device: state.selectedDevice.path,
                dest: destination,
                sign: true,
                signKeyPath: signKeyPath || undefined,
                includePaths,
                allowCritical: state.selectedDevice.risk_level === 'CRITICAL'
            });

            // Show completion state
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Backup Complete!',
                    currentStep: 5,
                    totalSteps: 5,
                    currentStepName: '✅ All operations completed successfully',
                    percentage: 100
                }
            });

            addToast('Backup completed successfully! 🎉', 'success');

            // Clear progress and navigate after showing completion
            setTimeout(() => {
                dispatch({ type: 'SET_PROGRESS', payload: null });
                navigate('/certificates');
            }, 3000); // Extended to 3 seconds to see completion
        } catch (error) {
            console.error('Backup failed:', error);
            addToast('Backup operation failed. Please check logs for details.', 'error');
            dispatch({ type: 'SET_PROGRESS', payload: null });
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
            <h1 className="text-2xl font-semibold mb-8">📦 Encrypted Backup</h1>

            {/* Device Info */}
            <div className="card mb-8">
                <h2 className="text-lg font-semibold mb-4">Source Device</h2>
                <div className="device-info">
                    <div className="device-detail">
                        <strong>Model:</strong> {state.selectedDevice.model}
                    </div>
                    <div className="device-detail">
                        <strong>Path:</strong> {state.selectedDevice.path}
                    </div>
                    <div className="device-detail">
                        <strong>Size:</strong> {(state.selectedDevice.capacity / (1024 ** 3)).toFixed(1)} GB
                    </div>
                    <div className="device-detail">
                        <strong>Risk Level:</strong> 
                        <span className={`risk-badge ${state.selectedDevice.risk_level.toLowerCase()}`}>
                            {state.selectedDevice.risk_level}
                        </span>
                    </div>
                </div>
            </div>

            {/* Critical Disk Warning */}
            {state.selectedDevice.risk_level === 'CRITICAL' && (
                <div className="alert alert-warning mb-8">
                    <div className="alert-icon">⚠️</div>
                    <div>
                        <h3 className="font-semibold mb-2">System Disk Warning</h3>
                        <p className="mb-3">
                            You are backing up from a system disk. For safety, only user directories will be included by default:
                        </p>
                        <ul className="mb-3">
                            <li>• Documents, Pictures, Videos, Music folders</li>
                            <li>• Desktop and Downloads folders</li>
                            <li>• System files and applications will be excluded</li>
                        </ul>
                        <p className="text-sm font-medium">
                            Use the file browser below for custom file selection if needed.
                        </p>
                    </div>
                </div>
            )}

            {/* Backup Configuration */}
            <div className="card mb-8">
                <h2 className="text-lg font-semibold mb-6">Backup Configuration</h2>

                {/* Destination */}
                <div className="form-group mb-6">
                    <label className="form-label">Backup Destination</label>
                    <div className="flex gap-3">
                        <input
                            type="text"
                            className="form-input flex-1"
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

                {/* Path Selection Mode */}
                {state.selectedDevice.risk_level === 'CRITICAL' && (
                    <div className="form-group mb-6">
                        <label className="form-label flex items-center gap-3">
                            <input
                                type="checkbox"
                                checked={useDefaultPaths}
                                onChange={(e) => setUseDefaultPaths(e.target.checked)}
                            />
                            Use default user directories (recommended for system disks)
                        </label>
                        <p className="form-help">
                            Safe selection of common user folders, excluding system files.
                        </p>
                    </div>
                )}

                {/* File Browser Section */}
                {!useDefaultPaths && (
                    <div className="mt-6">
                        <div className="flex items-center justify-between mb-4">
                            <div>
                                <h3 className="font-semibold">
                                    {state.selectedDevice.risk_level === 'CRITICAL' 
                                        ? 'Custom File Selection (Advanced)' 
                                        : 'Select Files and Folders to Backup'
                                    }
                                </h3>
                                <p className="text-sm text-gray-600">
                                    {state.selectedDevice.risk_level === 'CRITICAL' 
                                        ? 'Only select files outside default user directories if necessary.'
                                        : 'Browse and select specific files and folders for your backup.'
                                    }
                                </p>
                            </div>
                            <button
                                className="btn btn-primary"
                                onClick={() => setShowFileBrowser(!showFileBrowser)}
                            >
                                {showFileBrowser ? '🔽 Hide Browser' : '📁 Browse Files'}
                            </button>
                        </div>

                        {/* Selected Files Display */}
                        {selectedFiles.length > 0 && (
                            <div className="mb-4 p-4 bg-blue-50 rounded-lg border border-blue-200">
                                <h4 className="font-semibold text-blue-900 mb-2">
                                    {selectedFiles.length} Item{selectedFiles.length !== 1 ? 's' : ''} Selected
                                </h4>
                                <div className="max-h-32 overflow-y-auto space-y-1">
                                    {selectedFiles.map((path, index) => (
                                        <div key={index} className="text-sm font-mono text-blue-800">
                                            📄 {path}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* File Browser Component */}
                        {showFileBrowser && (
                            <div className="mb-6 p-4 bg-gray-50 rounded-lg border">
                                <h4 className="font-semibold mb-3">File Browser</h4>
                                <FileBrowser
                                    onSelectionChange={setSelectedFiles}
                                    multiSelect={true}
                                    allowFiles={true}
                                    allowFolders={true}
                                    maxSelectionSize={50 * 1024 * 1024 * 1024} // 50GB warning
                                    title="Select Files and Folders for Backup"
                                />
                            </div>
                        )}
                    </div>
                )}

                {/* Signing Key */}
                <div className="form-group mb-6">
                    <label className="form-label">Signing Key Path (Optional)</label>
                    <input
                        type="text"
                        className="form-input"
                        value={signKeyPath}
                        onChange={(e) => setSignKeyPath(e.target.value)}
                        placeholder="Path to private key for certificate signing"
                    />
                    <p className="form-help">
                        Leave empty to use default development key. Specify custom key for production use.
                    </p>
                </div>
            </div>

            {/* Action Button */}
            <div className="mb-8">
                <button
                    className="btn btn-primary btn-large"
                    onClick={handleRunBackup}
                    disabled={state.isLoading || running}
                    style={{ width: '100%' }}
                >
                    {(state.isLoading || running) ? '⏳ Running Backup...' : '🛡️ Start Encrypted Backup'}
                </button>
            </div>

            {/* Progress Indicator */}
            {state.isLoading && state.progress && (
                <Progress
                    title={state.progress.title}
                    currentStep={state.progress.currentStep}
                    totalSteps={state.progress.totalSteps}
                    currentStepName={state.progress.currentStepName}
                    percentage={state.progress.percentage}
                    className="mb-8"
                />
            )}

            {/* Operation Logs */}
            {state.logs.length > 0 && (
                <div className="mb-8">
                    <LogViewer
                        logs={state.logs}
                        title="Backup Progress"
                    />
                </div>
            )}

            {/* Backup Results */}
            {state.backupResult && (
                <div className="mb-8">
                    <div className="card">
                        <h2 className="text-lg font-semibold mb-4 text-green-600">
                            ✅ Backup Completed Successfully
                        </h2>

                        <div className="grid grid-cols-2 gap-6 mb-6">
                            <div>
                                <h3 className="font-semibold mb-3">Backup Summary</h3>
                                <div className="space-y-2">
                                    <div>
                                        <span className="font-medium">Location:</span>
                                        <div className="mt-1 p-2 bg-gray-100 rounded font-mono text-sm">
                                            {state.backupResult.backup_path}
                                        </div>
                                    </div>
                                    <div>
                                        <span className="font-medium">Integrity Checks:</span>
                                        <div className="mt-1 text-green-600">
                                            ✓ {state.backupResult.integrity_checks} samples verified
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div>
                                <h3 className="font-semibold mb-3">Generated Files</h3>
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
                    <div className="loading-spinner"></div>
                    <span>{state.currentOperation}</span>
                </div>
            )}
        </div>
    );
}

export default Backup;
