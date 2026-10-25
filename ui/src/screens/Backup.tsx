import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { open } from '@tauri-apps/api/dialog';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe } from '../hooks/useSecureWipe';
import LogViewer from '../components/LogViewer';
import FileLink from '../components/FileLink';
import Progress from '../components/Progress';
import FileBrowser from '../components/FileBrowser';
import {
    COMPLETE_PROGRESS,
    LAST_TIMER_STEP,
    failedProgress,
    friendlyBackupError,
    runningProgress,
    stepForLine,
} from '../lib/backupProgress';
import { createLogger } from '../lib/logger';

const log = createLogger('backup');

function Backup() {
    const navigate = useNavigate();
    const { state, addToast, dispatch, clearLogs } = useApp();
    const { backup, running } = useSecureWipe();
    const [destination, setDestination] = useState('~/SecureWipe/backups');
    const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
    const [signKeyPath, setSignKeyPath] = useState('');
    const [useDefaultPaths, setUseDefaultPaths] = useState(true);
    const [showFileBrowser, setShowFileBrowser] = useState(false);
    // A ref, not state: the handlers that stop the timer were created before it started,
    // so a state copy in their closure was always null and the timer never stopped.
    const progressTimer = useRef<ReturnType<typeof setInterval> | null>(null);
    const stopProgressTimer = () => {
        if (progressTimer.current) {
            clearInterval(progressTimer.current);
            progressTimer.current = null;
        }
    };
    const reachedStep = useRef(0);

    // Delayed follow-ups (clearing the bar, navigating) are cancelled if the screen unmounts,
    // and the bar is cleared then, since only this screen shows it.
    const pending = useRef<ReturnType<typeof setTimeout>[]>([]);
    const schedule = (fn: () => void, ms: number) => {
        pending.current.push(setTimeout(fn, ms));
    };
    useEffect(() => {
        const timers = pending.current;
        return () => {
            stopProgressTimer();
            timers.forEach(clearTimeout);
            dispatch({ type: 'SET_PROGRESS', payload: null });
        };
    }, [dispatch]);

    // Start with an empty log so output from device discovery does not read as backup progress.
    useEffect(() => {
        clearLogs();
    }, [clearLogs]);

    // While the backup runs, move the bar to the step the newest output line names.
    // Finishing is decided by the backup result in handleRunBackup, not by log lines.
    useEffect(() => {
        if (!running) return;
        const latest = state.logs[state.logs.length - 1];
        const step = latest ? stepForLine(latest.line) : null;
        if (step) {
            reachedStep.current = Math.max(reachedStep.current, step);
            dispatch({ type: 'SET_PROGRESS', payload: runningProgress(step) });
        }
    }, [state.logs, running, dispatch]);

    const handleSelectDestination = async () => {
        try {
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
            log.error('destination dialog failed', undefined, error);
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

        const device = state.selectedDevice;
        dispatch({ type: 'CLEAR_LOGS' });
        addToast(`Starting backup of ${device.model}...`, 'info');

        reachedStep.current = 1;
        dispatch({ type: 'SET_PROGRESS', payload: runningProgress(1) });

        // Output can go quiet for a long time, so nudge the bar forward every 8 seconds.
        stopProgressTimer();
        progressTimer.current = setInterval(() => {
            if (reachedStep.current < LAST_TIMER_STEP) {
                reachedStep.current += 1;
                dispatch({ type: 'SET_PROGRESS', payload: runningProgress(reachedStep.current) });
            }
        }, 8000);

        try {
            const result = await backup({
                device: device.path,
                dest: destination,
                sign: true,
                signKeyPath: signKeyPath || undefined,
                includePaths: !useDefaultPaths && selectedFiles.length > 0 ? selectedFiles : undefined,
                allowCritical: device.risk_level === 'CRITICAL'
            });
            stopProgressTimer();

            dispatch({ type: 'SET_PROGRESS', payload: COMPLETE_PROGRESS });
            let completionMessage = 'Backup completed successfully! 🎉';
            if (result.certPathJson) {
                completionMessage += ` Certificate saved to: ${result.certPathJson}`;
            }
            addToast(completionMessage, 'success');

            schedule(() => {
                dispatch({ type: 'SET_PROGRESS', payload: null });
                navigate('/certificates');
            }, 4000);
        } catch (error) {
            stopProgressTimer();
            log.error('backup failed', { device: device.path }, error);

            const errorMessage = friendlyBackupError(error);
            dispatch({ type: 'SET_PROGRESS', payload: failedProgress(errorMessage) });
            addToast(`Backup operation failed: ${errorMessage}`, 'error');

            // Clear the error after 10 seconds so the user can retry.
            schedule(() => {
                dispatch({ type: 'SET_PROGRESS', payload: null });
            }, 10000);
        }
    };

    const handleBackToDiscover = () => {
        navigate('/backup-discover');
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
                        Please discover and select a device for backup before running backup operations.
                    </p>
                    <button
                        className="btn btn-primary"
                        onClick={handleBackToDiscover}
                    >
                        🔍 Go to Backup Device Discovery
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
            {/* Polished Header Section */}
            <div className="mb-8">
                <div className="flex items-center gap-4 mb-4">
                    <div style={{ 
                        width: '56px', 
                        height: '56px', 
                        background: 'linear-gradient(135deg, #3b82f6 0%, #1e40af 100%)', 
                        borderRadius: '16px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontSize: '28px',
                        boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)'
                    }}>
                        🛡️
                    </div>
                    <div>
                        <h1 className="font-semibold" style={{ fontSize: '2.25rem', marginBottom: '0.5rem', color: '#1e293b' }}>
                            Encrypted Backup
                        </h1>
                        <p style={{ color: '#64748b', fontSize: '1.125rem' }}>
                            Secure your data with military-grade AES-256-CTR encryption
                        </p>
                    </div>
                </div>
                
                {/* Security Features Card */}
                <div className="alert alert-info" style={{ 
                    border: 'none', 
                    background: 'linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%)',
                    borderRadius: '16px',
                    padding: '1.5rem'
                }}>
                    <div className="flex items-start gap-4">
                        <div style={{ 
                            fontSize: '2rem',
                            background: 'linear-gradient(135deg, #3b82f6 0%, #1e40af 100%)',
                            width: '48px',
                            height: '48px',
                            borderRadius: '12px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center'
                        }}>
                            🔐
                        </div>
                        <div className="flex-1">
                            <h3 className="font-semibold mb-3" style={{ color: '#1e40af', fontSize: '1.25rem' }}>
                                Advanced Security Features
                            </h3>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="flex items-center gap-3">
                                    <div style={{ 
                                        width: '24px', 
                                        height: '24px', 
                                        background: '#10b981', 
                                        borderRadius: '50%',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '12px',
                                        color: 'white'
                                    }}>✓</div>
                                    <span style={{ fontWeight: '500' }}>AES-256-CTR Encryption</span>
                                </div>
                                <div className="flex items-center gap-3">
                                    <div style={{ 
                                        width: '24px', 
                                        height: '24px', 
                                        background: '#10b981', 
                                        borderRadius: '50%',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '12px',
                                        color: 'white'
                                    }}>✓</div>
                                    <span style={{ fontWeight: '500' }}>PBKDF2 Key Derivation</span>
                                </div>
                                <div className="flex items-center gap-3">
                                    <div style={{ 
                                        width: '24px', 
                                        height: '24px', 
                                        background: '#10b981', 
                                        borderRadius: '50%',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '12px',
                                        color: 'white'
                                    }}>✓</div>
                                    <span style={{ fontWeight: '500' }}>HMAC-SHA256 Integrity</span>
                                </div>
                                <div className="flex items-center gap-3">
                                    <div style={{ 
                                        width: '24px', 
                                        height: '24px', 
                                        background: '#10b981', 
                                        borderRadius: '50%',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '12px',
                                        color: 'white'
                                    }}>✓</div>
                                    <span style={{ fontWeight: '500' }}>N=5 Sample Verification</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Polished Device Info */}
            <div className="card mb-8" style={{ 
                background: 'linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%)', 
                border: '1px solid #e2e8f0',
                borderRadius: '16px',
                padding: '2rem'
            }}>
                <div className="flex items-center gap-4 mb-6">
                    <div style={{ 
                        width: '48px', 
                        height: '48px', 
                        background: state.selectedDevice.risk_level === 'CRITICAL' 
                            ? 'linear-gradient(135deg, #fee2e2 0%, #fecaca 100%)' 
                            : 'linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%)',
                        borderRadius: '12px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontSize: '24px'
                    }}>
                        💾
                    </div>
                    <div className="flex-1">
                        <h2 className="font-semibold mb-2" style={{ fontSize: '1.5rem', color: '#1e293b' }}>
                            Source Device
                        </h2>
                        <p style={{ color: '#64748b', fontSize: '1rem' }}>
                            The device that will be backed up before wiping
                        </p>
                    </div>
                    <div className={`risk-badge ${state.selectedDevice.risk_level.toLowerCase()}`} style={{
                        fontSize: '0.875rem',
                        fontWeight: '600',
                        padding: '0.5rem 1rem',
                        borderRadius: '8px'
                    }}>
                        {state.selectedDevice.risk_level}
                    </div>
                </div>
                
                <div className="grid grid-cols-3 gap-6">
                    <div className="text-center p-6 bg-white rounded-xl border border-gray-100" style={{ boxShadow: '0 1px 3px 0 rgb(0 0 0 / 0.1)' }}>
                        <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🏷️</div>
                        <div className="font-semibold text-gray-900" style={{ fontSize: '1.125rem', marginBottom: '0.5rem' }}>
                            {state.selectedDevice.model}
                        </div>
                        <div className="text-sm text-gray-500">Device Model</div>
                    </div>
                    <div className="text-center p-6 bg-white rounded-xl border border-gray-100" style={{ boxShadow: '0 1px 3px 0 rgb(0 0 0 / 0.1)' }}>
                        <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>📊</div>
                        <div className="font-semibold text-gray-900" style={{ fontSize: '1.125rem', marginBottom: '0.5rem' }}>
                            {(state.selectedDevice.capacity / (1024 ** 3)).toFixed(1)} GB
                        </div>
                        <div className="text-sm text-gray-500">Storage Capacity</div>
                    </div>
                    <div className="text-center p-6 bg-white rounded-xl border border-gray-100" style={{ boxShadow: '0 1px 3px 0 rgb(0 0 0 / 0.1)' }}>
                        <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🔢</div>
                        <div className="font-semibold text-gray-900" style={{ 
                            fontSize: '0.8rem', 
                            marginBottom: '0.5rem', 
                            fontFamily: 'monospace',
                            wordBreak: 'break-all',
                            lineHeight: '1.2'
                        }}>
                            {state.selectedDevice.serial}
                        </div>
                        <div className="text-sm text-gray-500">Serial Number</div>
                    </div>
                </div>
            </div>

            {/* Polished Backup Configuration */}
            <div className="card mb-8" style={{ 
                background: '#ffffff', 
                border: '1px solid #e2e8f0',
                borderRadius: '16px',
                padding: '2rem'
            }}>
                <div className="flex items-center gap-4 mb-6">
                    <div style={{ 
                        width: '48px', 
                        height: '48px', 
                        background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                        borderRadius: '12px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontSize: '24px'
                    }}>
                        ⚙️
                    </div>
                    <div>
                        <h2 className="font-semibold" style={{ fontSize: '1.5rem', marginBottom: '0.5rem', color: '#1e293b' }}>
                            Backup Configuration
                        </h2>
                        <p style={{ color: '#64748b', fontSize: '1rem' }}>
                            Configure your backup settings and select files to protect
                        </p>
                    </div>
                </div>

                {/* Critical Disk Warning */}
                {state.selectedDevice.risk_level === 'CRITICAL' && (
                    <div className="alert alert-warning mb-6" style={{ 
                        background: 'linear-gradient(135deg, #fef3c7 0%, #fde68a 100%)',
                        border: '1px solid #f59e0b',
                        borderRadius: '12px',
                        padding: '1.5rem'
                    }}>
                        <div className="flex items-start gap-4">
                            <div style={{ fontSize: '2.5rem' }}>🖥️</div>
                            <div className="flex-1">
                                <h3 className="font-semibold mb-3" style={{ color: '#92400e', fontSize: '1.25rem' }}>
                                    System Disk Backup Mode
                                </h3>
                                <p className="mb-4" style={{ color: '#78350f' }}>
                                    Backing up from system disk. Only user files will be included by default for safety:
                                </p>
                                <div className="grid grid-cols-2 gap-3 mb-4">
                                    {[
                                        'Documents folder',
                                        'Pictures folder', 
                                        'Videos folder',
                                        'Music folder',
                                        'Desktop folder',
                                        'Downloads folder'
                                    ].map((folder, index) => (
                                        <div key={index} className="flex items-center gap-2">
                                            <span style={{ color: '#16a34a', fontSize: '1.125rem' }}>✓</span>
                                            <span className="text-sm font-medium">{folder}</span>
                                        </div>
                                    ))}
                                </div>
                                <div className="flex items-center gap-3 p-3 bg-white bg-opacity-60 rounded-lg">
                                    <span style={{ color: '#dc2626', fontSize: '1.25rem' }}>⚠️</span>
                                    <span className="text-sm font-medium" style={{ color: '#78350f' }}>
                                        System files and applications will be excluded automatically for security.
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {/* Destination Configuration */}
                <div className="form-group mb-6">
                    <label className="form-label" style={{ fontSize: '1.125rem', fontWeight: '600', marginBottom: '0.75rem' }}>
                        Backup Destination
                    </label>
                    <div className="flex gap-3">
                        <input
                            type="text"
                            className="form-input flex-1"
                            value={destination}
                            onChange={(e) => setDestination(e.target.value)}
                            placeholder="~/SecureWipe/backups"
                            style={{ 
                                padding: '0.875rem 1rem',
                                fontSize: '1rem',
                                borderRadius: '8px',
                                border: '2px solid #e5e7eb',
                                transition: 'border-color 0.2s ease'
                            }}
                        />
                        <button
                            className="btn btn-secondary"
                            onClick={handleSelectDestination}
                            style={{ 
                                padding: '0.875rem 1.5rem',
                                borderRadius: '8px',
                                fontWeight: '600'
                            }}
                        >
                            📁 Browse
                        </button>
                    </div>
                </div>

                {/* Path Selection Mode */}
                {state.selectedDevice.risk_level === 'CRITICAL' && (
                    <div className="form-group mb-6">
                        <label className="form-label flex items-center gap-3" style={{ fontSize: '1rem', cursor: 'pointer' }}>
                            <input
                                type="checkbox"
                                checked={useDefaultPaths}
                                onChange={(e) => setUseDefaultPaths(e.target.checked)}
                                style={{ width: '1.25rem', height: '1.25rem' }}
                            />
                            <span className="font-semibold">Use default user directories (recommended)</span>
                        </label>
                        <p style={{ color: '#64748b', fontSize: '0.875rem', marginTop: '0.5rem', marginLeft: '2rem' }}>
                            Safe selection of common user folders, excluding system files.
                        </p>
                    </div>
                )}

                {/* File Browser Section */}
                {!useDefaultPaths && (
                    <div className="mt-6">
                        <div className="flex items-center justify-between mb-4">
                            <div>
                                <h3 className="font-semibold" style={{ fontSize: '1.25rem', marginBottom: '0.5rem' }}>
                                    {state.selectedDevice.risk_level === 'CRITICAL' 
                                        ? 'Custom Additional Files/Folders' 
                                        : 'Select Files and Folders to Backup'
                                    }
                                </h3>
                                <p style={{ color: '#64748b', fontSize: '0.875rem' }}>
                                    {state.selectedDevice.risk_level === 'CRITICAL' 
                                        ? 'Advanced: Only select files outside the default user directories if absolutely necessary.'
                                        : 'Browse and select specific files and folders for your backup.'
                                    }
                                </p>
                            </div>
                            <button
                                className="btn btn-primary"
                                onClick={() => setShowFileBrowser(!showFileBrowser)}
                                style={{ 
                                    background: showFileBrowser 
                                        ? 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)'
                                        : 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
                                    minWidth: '160px',
                                    borderRadius: '8px',
                                    fontWeight: '600'
                                }}
                            >
                                {showFileBrowser ? '📁 Hide Browser' : '📁 Browse Files'}
                            </button>
                        </div>

                        {/* Selected Files Display */}
                        {selectedFiles.length > 0 && (
                            <div className="mb-4 p-4 bg-blue-50 rounded-xl border border-blue-200">
                                <div className="flex items-center gap-3 mb-3">
                                    <div style={{ 
                                        width: '36px', 
                                        height: '36px', 
                                        background: 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
                                        borderRadius: '10px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '18px'
                                    }}>
                                        📄
                                    </div>
                                    <div>
                                        <h4 className="font-semibold text-blue-900" style={{ fontSize: '1.125rem' }}>
                                            {selectedFiles.length} Item{selectedFiles.length !== 1 ? 's' : ''} Selected
                                        </h4>
                                        <p className="text-sm text-blue-700">
                                            These files and folders will be included in your backup
                                        </p>
                                    </div>
                                </div>
                                <div className="max-h-32 overflow-y-auto space-y-2">
                                    {selectedFiles.map((path, index) => (
                                        <div key={index} className="flex items-center gap-3 text-sm bg-white bg-opacity-70 p-3 rounded-lg">
                                            <span style={{ color: '#3b82f6', fontSize: '1rem' }}>📄</span>
                                            <span className="font-mono text-gray-700 truncate flex-1" style={{ fontSize: '0.875rem' }}>
                                                {path}
                                            </span>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* File Browser Component */}
                        {showFileBrowser && (
                            <div className="mb-6 p-6 bg-gray-50 rounded-xl border border-gray-200">
                                <div className="flex items-center gap-4 mb-4">
                                    <div style={{ 
                                        width: '36px', 
                                        height: '36px', 
                                        background: 'linear-gradient(135deg, #6366f1 0%, #4f46e5 100%)',
                                        borderRadius: '10px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '18px'
                                    }}>
                                        🗂️
                                    </div>
                                    <div>
                                        <h4 className="font-semibold text-gray-900" style={{ fontSize: '1.125rem' }}>
                                            File Browser
                                        </h4>
                                        <p className="text-sm text-gray-600">
                                            Navigate and select files and folders for your backup
                                        </p>
                                    </div>
                                </div>
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

                {/* Signing Key Configuration */}
                <div className="form-group mb-6">
                    <label className="form-label" style={{ fontSize: '1.125rem', fontWeight: '600', marginBottom: '0.75rem' }}>
                        Signing Key Path (Optional)
                    </label>
                    <input
                        type="text"
                        className="form-input"
                        value={signKeyPath}
                        onChange={(e) => setSignKeyPath(e.target.value)}
                        placeholder="Path to private key for certificate signing"
                        style={{ 
                            padding: '0.875rem 1rem',
                            fontSize: '1rem',
                            borderRadius: '8px',
                            border: '2px solid #e5e7eb'
                        }}
                    />
                    <p style={{ color: '#64748b', fontSize: '0.875rem', marginTop: '0.5rem' }}>
                        Leave empty to use default development key. Specify custom key for production use.
                    </p>
                </div>
            </div>

            {/* Polished Action Button */}
            <div className="mb-8">
                <button
                    className="btn btn-primary btn-large"
                    onClick={handleRunBackup}
                    disabled={running}
                    style={{ 
                        width: '100%',
                        background: running 
                            ? 'linear-gradient(135deg, #6b7280 0%, #4b5563 100%)'
                            : 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                        fontSize: '1.25rem',
                        fontWeight: '700',
                        padding: '1.25rem 2rem',
                        borderRadius: '16px',
                        boxShadow: '0 8px 25px -8px rgba(16, 185, 129, 0.5)',
                        transition: 'all 0.3s ease',
                        border: 'none'
                    }}
                >
                    {running ? (
                        <div className="flex items-center justify-center gap-3">
                            <div style={{
                                width: '24px',
                                height: '24px',
                                border: '3px solid #ffffff',
                                borderTopColor: 'transparent',
                                borderRadius: '50%',
                                animation: 'spin 1s linear infinite'
                            }}></div>
                            <span>
                                {state.progress?.percentage === 100 ? 'Completing...' : 'Running Backup...'}
                            </span>
                        </div>
                    ) : (
                        <div className="flex items-center justify-center gap-4">
                            <span style={{ fontSize: '1.5rem' }}>🛡️</span>
                            <span>Start Encrypted Backup</span>
                        </div>
                    )}
                </button>
            </div>

            {/* Progress Indicator */}
            {state.progress && (
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
                    <div className="card" style={{ borderRadius: '16px', padding: '2rem' }}>
                        <div className="flex items-center gap-4 mb-6">
                            <div style={{ 
                                width: '48px', 
                                height: '48px', 
                                background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                                borderRadius: '12px',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                fontSize: '24px'
                            }}>
                                ✅
                            </div>
                            <div>
                                <h2 className="font-semibold" style={{ fontSize: '1.5rem', color: '#059669' }}>
                                    Backup Completed Successfully
                                </h2>
                                <p style={{ color: '#64748b' }}>
                                    Your data has been securely encrypted and backed up
                                </p>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-6 mb-6">
                            <div>
                                <h3 className="font-semibold mb-3" style={{ fontSize: '1.125rem' }}>Backup Summary</h3>
                                <div className="space-y-3">
                                    <div>
                                        <span className="font-medium">Location:</span>
                                        <div className="mt-1 p-2 bg-gray-50 rounded font-mono text-sm">
                                            {state.backupResult.backup_path}
                                        </div>
                                    </div>
                                    <div>
                                        <span className="font-medium">Integrity Checks:</span>
                                        <div className="mt-1 flex items-center gap-2">
                                            <span style={{ color: '#10b981', fontSize: '1.125rem' }}>✓</span>
                                            <span>{state.backupResult.integrity_checks} samples verified</span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div>
                                <h3 className="font-semibold mb-3" style={{ fontSize: '1.125rem' }}>Generated Files</h3>
                                <div className="space-y-3">
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
                                style={{ borderRadius: '8px', fontWeight: '600' }}
                            >
                                📜 View All Certificates
                            </button>

                            <button
                                className="btn btn-secondary"
                                onClick={() => window.open('http://localhost:8000', '_blank')}
                                style={{ borderRadius: '8px', fontWeight: '600' }}
                            >
                                🌐 Open in Portal Verify
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Current Operation Status */}
            {state.currentOperation && (
                <div className="alert alert-info" style={{ borderRadius: '12px' }}>
                    <div className="flex items-center gap-3">
                        <div style={{
                            width: '24px',
                            height: '24px',
                            border: '3px solid #3b82f6',
                            borderTopColor: 'transparent',
                            borderRadius: '50%',
                            animation: 'spin 1s linear infinite'
                        }}></div>
                        <span style={{ fontWeight: '500' }}>{state.currentOperation}</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default Backup;
