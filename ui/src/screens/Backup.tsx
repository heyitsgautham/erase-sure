import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { open } from '@tauri-apps/api/dialog';
import { useApp } from '../contexts/AppContext';
import { useBackupRun } from '../hooks/useBackupRun';
import LogViewer from '../components/LogViewer';
import Progress from '../components/Progress';
import BackupConfigForm from '../components/backup/BackupConfigForm';
import BackupHeader from '../components/backup/BackupHeader';
import BackupResultCard from '../components/backup/BackupResultCard';
import SourceDeviceCard from '../components/backup/SourceDeviceCard';
import { createLogger } from '../lib/logger';

const log = createLogger('backup');

function Backup() {
    const navigate = useNavigate();
    const { state, addToast } = useApp();
    const { running, startBackup } = useBackupRun();
    const [destination, setDestination] = useState('~/SecureWipe/backups');
    const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
    const [signKeyPath, setSignKeyPath] = useState('');
    const [useDefaultPaths, setUseDefaultPaths] = useState(true);
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

    const handleRunBackup = () => {
        if (!state.selectedDevice) {
            addToast('Please select a device first', 'error');
            return;
        }
        void startBackup({
            device: state.selectedDevice,
            destination,
            signKeyPath,
            includePaths: !useDefaultPaths && selectedFiles.length > 0 ? selectedFiles : undefined,
        });
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
            <BackupHeader />

            <SourceDeviceCard device={state.selectedDevice} />

            <BackupConfigForm
                device={state.selectedDevice}
                destination={destination}
                onDestinationChange={setDestination}
                onBrowseDestination={handleSelectDestination}
                useDefaultPaths={useDefaultPaths}
                onUseDefaultPathsChange={setUseDefaultPaths}
                selectedFiles={selectedFiles}
                onSelectedFilesChange={setSelectedFiles}
                signKeyPath={signKeyPath}
                onSignKeyPathChange={setSignKeyPath}
            />

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
                    <BackupResultCard result={state.backupResult} onViewCertificates={handleViewCertificates} />
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
