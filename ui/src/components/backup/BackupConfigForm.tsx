import { useState } from 'react';
import type { Device } from '../../contexts/AppContext';
import FileBrowser from '../FileBrowser';

interface BackupConfigFormProps {
    device: Device;
    destination: string;
    onDestinationChange: (destination: string) => void;
    onBrowseDestination: () => void;
    useDefaultPaths: boolean;
    onUseDefaultPathsChange: (useDefaults: boolean) => void;
    selectedFiles: string[];
    onSelectedFilesChange: (paths: string[]) => void;
    signKeyPath: string;
    onSignKeyPathChange: (path: string) => void;
}

/** Destination, what to include, and the signing key for one backup. */
function BackupConfigForm({
    device,
    destination,
    onDestinationChange,
    onBrowseDestination,
    useDefaultPaths,
    onUseDefaultPathsChange,
    selectedFiles,
    onSelectedFilesChange,
    signKeyPath,
    onSignKeyPathChange,
}: BackupConfigFormProps) {
    const [showFileBrowser, setShowFileBrowser] = useState(false);

    return (
        <div
            className="card mb-8"
            style={{
                background: '#ffffff',
                border: '1px solid #e2e8f0',
                borderRadius: '16px',
                padding: '2rem',
            }}
        >
            <div className="flex items-center gap-4 mb-6">
                <div
                    style={{
                        width: '48px',
                        height: '48px',
                        background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                        borderRadius: '12px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontSize: '24px',
                    }}
                >
                    ⚙️
                </div>
                <div>
                    <h2
                        className="font-semibold"
                        style={{ fontSize: '1.5rem', marginBottom: '0.5rem', color: '#1e293b' }}
                    >
                        Backup Configuration
                    </h2>
                    <p style={{ color: '#64748b', fontSize: '1rem' }}>
                        Configure your backup settings and select files to protect
                    </p>
                </div>
            </div>

            {/* Critical Disk Warning */}
            {device.risk_level === 'CRITICAL' && (
                <div
                    className="alert alert-warning mb-6"
                    style={{
                        background: 'linear-gradient(135deg, #fef3c7 0%, #fde68a 100%)',
                        border: '1px solid #f59e0b',
                        borderRadius: '12px',
                        padding: '1.5rem',
                    }}
                >
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
                                    'Downloads folder',
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
                <label
                    className="form-label"
                    style={{ fontSize: '1.125rem', fontWeight: '600', marginBottom: '0.75rem' }}
                >
                    Backup Destination
                </label>
                <div className="flex gap-3">
                    <input
                        type="text"
                        className="form-input flex-1"
                        value={destination}
                        onChange={(e) => onDestinationChange(e.target.value)}
                        placeholder="~/SecureWipe/backups"
                        style={{
                            padding: '0.875rem 1rem',
                            fontSize: '1rem',
                            borderRadius: '8px',
                            border: '2px solid #e5e7eb',
                            transition: 'border-color 0.2s ease',
                        }}
                    />
                    <button
                        className="btn btn-secondary"
                        onClick={onBrowseDestination}
                        style={{
                            padding: '0.875rem 1.5rem',
                            borderRadius: '8px',
                            fontWeight: '600',
                        }}
                    >
                        📁 Browse
                    </button>
                </div>
            </div>

            {/* Path Selection Mode */}
            {device.risk_level === 'CRITICAL' && (
                <div className="form-group mb-6">
                    <label
                        className="form-label flex items-center gap-3"
                        style={{ fontSize: '1rem', cursor: 'pointer' }}
                    >
                        <input
                            type="checkbox"
                            checked={useDefaultPaths}
                            onChange={(e) => onUseDefaultPathsChange(e.target.checked)}
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
                                {device.risk_level === 'CRITICAL'
                                    ? 'Custom Additional Files/Folders'
                                    : 'Select Files and Folders to Backup'}
                            </h3>
                            <p style={{ color: '#64748b', fontSize: '0.875rem' }}>
                                {device.risk_level === 'CRITICAL'
                                    ? 'Advanced: Only select files outside the default user directories if absolutely necessary.'
                                    : 'Browse and select specific files and folders for your backup.'}
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
                                fontWeight: '600',
                            }}
                        >
                            {showFileBrowser ? '📁 Hide Browser' : '📁 Browse Files'}
                        </button>
                    </div>

                    {/* Selected Files Display */}
                    {selectedFiles.length > 0 && (
                        <div className="mb-4 p-4 bg-blue-50 rounded-xl border border-blue-200">
                            <div className="flex items-center gap-3 mb-3">
                                <div
                                    style={{
                                        width: '36px',
                                        height: '36px',
                                        background: 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
                                        borderRadius: '10px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '18px',
                                    }}
                                >
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
                                    <div
                                        key={index}
                                        className="flex items-center gap-3 text-sm bg-white bg-opacity-70 p-3 rounded-lg"
                                    >
                                        <span style={{ color: '#3b82f6', fontSize: '1rem' }}>📄</span>
                                        <span
                                            className="font-mono text-gray-700 truncate flex-1"
                                            style={{ fontSize: '0.875rem' }}
                                        >
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
                                <div
                                    style={{
                                        width: '36px',
                                        height: '36px',
                                        background: 'linear-gradient(135deg, #6366f1 0%, #4f46e5 100%)',
                                        borderRadius: '10px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontSize: '18px',
                                    }}
                                >
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
                                onSelectionChange={onSelectedFilesChange}
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
                <label
                    className="form-label"
                    style={{ fontSize: '1.125rem', fontWeight: '600', marginBottom: '0.75rem' }}
                >
                    Signing Key Path (Optional)
                </label>
                <input
                    type="text"
                    className="form-input"
                    value={signKeyPath}
                    onChange={(e) => onSignKeyPathChange(e.target.value)}
                    placeholder="Path to private key for certificate signing"
                    style={{
                        padding: '0.875rem 1rem',
                        fontSize: '1rem',
                        borderRadius: '8px',
                        border: '2px solid #e5e7eb',
                    }}
                />
                <p style={{ color: '#64748b', fontSize: '0.875rem', marginTop: '0.5rem' }}>
                    Leave empty to use default development key. Specify custom key for production use.
                </p>
            </div>
        </div>
    );
}

export default BackupConfigForm;
