import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
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
    const [progressTimer, setProgressTimer] = useState<ReturnType<typeof setInterval> | null>(null);
    const [completionProcessed, setCompletionProcessed] = useState(false);

    // Progress tracking based on log patterns - this runs for both running and completed states
    useEffect(() => {
        // Early return if no logs
        if (state.logs.length === 0) return;

        const latestLog = state.logs[state.logs.length - 1];
        if (!latestLog) return;

        // Check for completion patterns in recent logs
        const recentLogs = state.logs.slice(-5).join(' ');
        const completionPatterns = [
            /Backup completed successfully/i,
            /backup_complete/i,
            /Certificate saved to:/i,
            /Verification status: PASSED/i,
            /backup operation completed successfully/i
        ];

        const isCompleted = completionPatterns.some(pattern => pattern.test(recentLogs));

        // Handle completion - this works whether running is true or false
        if (isCompleted && !completionProcessed) {
            setCompletionProcessed(true);
            
            // Clear any existing progress timer
            if (progressTimer) {
                clearInterval(progressTimer);
                setProgressTimer(null);
            }

            // Mark as completed
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
            
            // Clear progress after showing completion and navigate
            setTimeout(() => {
                dispatch({ type: 'SET_PROGRESS', payload: null });
                // Also navigate to certificates if the backup succeeded
                navigate('/certificates');
            }, 3000);
            
            return;
        }

        // Only do regular progress tracking if we're actually running
        if (!running) return;

        const progressPatterns = [
            { pattern: /starting|initializing|begin|backup_start/i, step: 1, name: 'Initializing backup process...' },
            { pattern: /encryption|aes|key|crypto|cipher|backup_dir_created/i, step: 2, name: 'Setting up AES-256 encryption...' },
            { pattern: /copying|copy|encrypt|reading|writing|files|file_processing|manifest_created/i, step: 3, name: 'Copying and encrypting files...' },
            { pattern: /verifying|verify|check|integrity|hash|sample|verification_complete/i, step: 4, name: 'Verifying backup integrity...' },
            { pattern: /certificate_created|backup_complete|sign|complete|finished|done|success/i, step: 5, name: 'Generating certificates...' }
        ];

        // Only do additional completion checking if still running
        if (running) {
            const runningCompletionPatterns = [
                /Backup completed successfully/i,
                /backup_complete/i,
                /Certificate saved to:/i,
                /Verification status: PASSED/i,
                /backup operation completed successfully/i
            ];

            const runningIsCompleted = runningCompletionPatterns.some(pattern => pattern.test(latestLog));

            if (runningIsCompleted) {
                // Mark as completed - this will stop the running state
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
                
                // Clear the running state after a delay to show completion
                setTimeout(() => {
                    if (progressTimer) {
                        clearInterval(progressTimer);
                        setProgressTimer(null);
                    }
                }, 1000);
                
                return;
            }
        }

        // Regular progress tracking
        for (const { pattern, step, name } of progressPatterns) {
            if (pattern.test(latestLog)) {
                const percentage = Math.min(100, (step / 5) * 100);
                dispatch({
                    type: 'SET_PROGRESS',
                    payload: {
                        title: running ? 'Encrypted Backup in Progress' : 'Backup Complete!',
                        currentStep: step,
                        totalSteps: 5,
                        currentStepName: name,
                        percentage
                    }
                });
                break;
            }
        }
    }, [state.logs, running, dispatch]);

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
            // Clear any previous logs and reset progress
            dispatch({ type: 'CLEAR_LOGS' });
            dispatch({ type: 'SET_PROGRESS', payload: null });
            setCompletionProcessed(false);

            addToast(`Starting backup of ${state.selectedDevice.model}...`, 'info');

            // Initialize progress
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Encrypted Backup in Progress',
                    currentStep: 1,
                    totalSteps: 5,
                    currentStepName: 'Initializing backup process...',
                    percentage: 20
                }
            });

            const includePaths = !useDefaultPaths && selectedFiles.length > 0 
                ? selectedFiles 
                : undefined;

            // Set up a fallback progress timer in case logs don't trigger progress updates
            let progressStep = 1;
            const timer = setInterval(() => {
                if (progressStep < 4) { // Don't auto-complete final step
                    progressStep++;
                    const progressMessages = [
                        'Initializing backup process...',
                        'Setting up AES-256 encryption...',
                        'Copying and encrypting files...',
                        'Verifying backup integrity...',
                        'Generating certificates...'
                    ];
                    
                    dispatch({
                        type: 'SET_PROGRESS',
                        payload: {
                            title: 'Encrypted Backup in Progress',
                            currentStep: progressStep,
                            totalSteps: 5,
                            currentStepName: progressMessages[progressStep - 1],
                            percentage: (progressStep / 5) * 100
                        }
                    });
                }
            }, 8000); // Update every 8 seconds as fallback
            
            setProgressTimer(timer);

            // Call the real backup function - this will stream progress via logs
            const result = await backup({
                device: state.selectedDevice.path,
                dest: destination,
                sign: true,
                signKeyPath: signKeyPath || undefined,
                includePaths,
                allowCritical: state.selectedDevice.risk_level === 'CRITICAL'
            });

            // Clear the fallback timer
            if (progressTimer) {
                clearInterval(progressTimer);
                setProgressTimer(null);
            }

            // Show completion state - the useSecureWipe hook should have already stopped running
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

            // Show detailed completion message
            let completionMessage = 'Backup completed successfully! 🎉';
            if (result.certPathJson) {
                completionMessage += ` Certificate saved to: ${result.certPathJson}`;
            }
            addToast(completionMessage, 'success');

            // Clear progress and navigate after showing completion
            setTimeout(() => {
                dispatch({ type: 'SET_PROGRESS', payload: null });
                navigate('/certificates');
            }, 4000); // Extended to show completion message

        } catch (error) {
            // Clear the fallback timer on error if it exists
            if (progressTimer) {
                clearInterval(progressTimer);
                setProgressTimer(null);
            }
            
            console.error('Backup failed:', error);
            let errorMessage = 'Unknown error occurred';
            
            if (error instanceof Error) {
                errorMessage = error.message;
                // Check for common errors
                if (error.message.includes('securewipe')) {
                    errorMessage = 'SecureWipe CLI not found. Please ensure the core backup tool is built and accessible.';
                } else if (error.message.includes('permission')) {
                    errorMessage = 'Permission denied. Please check file/folder permissions for the backup destination.';
                } else if (error.message.includes('No such file')) {
                    errorMessage = 'Source device or destination path not found. Please verify the paths exist.';
                } else if (error.message.includes('timeout') || error.message.includes('Process timeout')) {
                    errorMessage = 'Backup operation timed out. This may happen with very large files. Please try with smaller folders or check system resources.';
                }
            }
            
            // Show error state instead of crashing
            dispatch({
                type: 'SET_PROGRESS',
                payload: {
                    title: 'Backup Failed',
                    currentStep: 0,
                    totalSteps: 5,
                    currentStepName: `❌ Error: ${errorMessage}`,
                    percentage: 0
                }
            });
            
            addToast(`Backup operation failed: ${errorMessage}`, 'error');
            
            // Clear error state after 10 seconds to allow retry
            setTimeout(() => {
                dispatch({ type: 'SET_PROGRESS', payload: null });
            }, 10000);
            
            // Don't let the error crash the app - just log it and continue
            console.warn('Backup error handled gracefully:', error);
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
                        <div className="font-semibold text-gray-900" style={{ fontSize: '0.975rem', marginBottom: '0.5rem', fontFamily: 'monospace' }}>
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
                    disabled={state.isLoading || running}
                    style={{ 
                        width: '100%',
                        background: (state.isLoading || running) 
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
                    {(state.isLoading || running) ? (
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
