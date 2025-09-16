import { useState } from 'react';
import { useSecureWipe } from '../hooks/useSecureWipe';
import LogViewer from '../components/LogViewer';
import type { Device, WipePlan, BackupResult } from '../types/securewipe';

export function SecureWipeDemo() {
    const { logs, running, discover, planWipe, backup, cancel, clearLogs } = useSecureWipe();
    const [devices, setDevices] = useState<Device[]>([]);
    const [selectedDevice, setSelectedDevice] = useState<string>('');
    const [wipePlan, setWipePlan] = useState<WipePlan | null>(null);
    const [backupResult, setBackupResult] = useState<BackupResult | null>(null);
    const [backupDest, setBackupDest] = useState<string>('');

    const handleDiscover = async () => {
        try {
            clearLogs();
            const foundDevices = await discover();
            setDevices(foundDevices);
        } catch (error) {
            console.error('Discover failed:', error);
        }
    };

    const handlePlanWipe = async () => {
        if (!selectedDevice) return;

        try {
            clearLogs();
            const plan = await planWipe({
                device: selectedDevice,
                samples: 128
            });
            setWipePlan(plan);
        } catch (error) {
            console.error('Plan wipe failed:', error);
        }
    };

    const handleBackup = async () => {
        if (!selectedDevice || !backupDest) return;

        try {
            clearLogs();
            const result = await backup({
                device: selectedDevice,
                dest: backupDest,
                sign: true,
            });
            setBackupResult(result);
        } catch (error) {
            console.error('Backup failed:', error);
        }
    };

    const handleCancel = async () => {
        try {
            await cancel();
        } catch (error) {
            console.error('Cancel failed:', error);
        }
    };

    return (
        <div className="p-6 max-w-6xl mx-auto">
            <h1 className="text-2xl font-bold mb-6">SecureWipe Demo</h1>

            {/* Control Panel */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
                <div className="space-y-4">
                    <div>
                        <h3 className="text-lg font-semibold mb-2">Device Discovery</h3>
                        <button
                            onClick={handleDiscover}
                            disabled={running}
                            className="btn btn-primary"
                        >
                            {running ? 'Discovering...' : 'Discover Devices'}
                        </button>
                    </div>

                    {devices.length > 0 && (
                        <div>
                            <h3 className="text-lg font-semibold mb-2">Select Device</h3>
                            <select
                                value={selectedDevice}
                                onChange={(e) => setSelectedDevice(e.target.value)}
                                className="w-full p-2 border rounded"
                            >
                                <option value="">Select a device...</option>
                                {devices.map((device) => (
                                    <option key={device.path} value={device.path}>
                                        {device.path} - {device.model} ({device.risk_level})
                                    </option>
                                ))}
                            </select>
                        </div>
                    )}

                    {selectedDevice && (
                        <div className="space-y-2">
                            <button
                                onClick={handlePlanWipe}
                                disabled={running}
                                className="btn btn-secondary w-full"
                            >
                                {running ? 'Planning...' : 'Create Wipe Plan'}
                            </button>

                            <div className="flex gap-2">
                                <input
                                    type="text"
                                    placeholder="Backup destination path"
                                    value={backupDest}
                                    onChange={(e) => setBackupDest(e.target.value)}
                                    className="flex-1 p-2 border rounded"
                                />
                                <button
                                    onClick={handleBackup}
                                    disabled={running || !backupDest}
                                    className="btn btn-secondary"
                                >
                                    {running ? 'Backing up...' : 'Backup'}
                                </button>
                            </div>
                        </div>
                    )}

                    {running && (
                        <button
                            onClick={handleCancel}
                            className="btn btn-danger"
                        >
                            Cancel Operation
                        </button>
                    )}
                </div>

                {/* Results Panel */}
                <div className="space-y-4">
                    {devices.length > 0 && (
                        <div>
                            <h3 className="text-lg font-semibold mb-2">Discovered Devices</h3>
                            <div className="space-y-2">
                                {devices.map((device) => (
                                    <div key={device.path} className="p-3 border rounded">
                                        <div className="font-mono text-sm">{device.path}</div>
                                        <div className="text-sm">{device.model}</div>
                                        <div className="text-xs text-gray-600">
                                            Risk: {device.risk_level} | Bus: {device.bus}
                                            {device.blocked && <span className="text-red-600"> | BLOCKED</span>}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {wipePlan && (
                        <div>
                            <h3 className="text-lg font-semibold mb-2">Wipe Plan</h3>
                            <div className="p-3 border rounded">
                                <div><strong>Policy:</strong> {wipePlan.policy}</div>
                                <div><strong>Method:</strong> {wipePlan.main_method}</div>
                                <div><strong>HPA/DCO Clear:</strong> {wipePlan.hpa_dco_clear ? 'Yes' : 'No'}</div>
                                <div><strong>Samples:</strong> {wipePlan.verification.samples}</div>
                                {wipePlan.blocked && (
                                    <div className="text-red-600"><strong>Blocked:</strong> {wipePlan.block_reason}</div>
                                )}
                            </div>
                        </div>
                    )}

                    {backupResult && (
                        <div>
                            <h3 className="text-lg font-semibold mb-2">Backup Result</h3>
                            <div className="p-3 border rounded space-y-1">
                                {backupResult.certPathJson && (
                                    <div className="text-sm">
                                        <strong>Certificate JSON:</strong> {backupResult.certPathJson}
                                    </div>
                                )}
                                {backupResult.certPathPdf && (
                                    <div className="text-sm">
                                        <strong>Certificate PDF:</strong> {backupResult.certPathPdf}
                                    </div>
                                )}
                                {backupResult.manifestSha256 && (
                                    <div className="text-sm">
                                        <strong>Manifest SHA256:</strong> {backupResult.manifestSha256}
                                    </div>
                                )}
                                {backupResult.filesProcessed && (
                                    <div className="text-sm">
                                        <strong>Files Processed:</strong> {backupResult.filesProcessed}
                                    </div>
                                )}
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Log Viewer */}
            <LogViewer logs={logs} title="SecureWipe Operation Logs" />
        </div>
    );
}