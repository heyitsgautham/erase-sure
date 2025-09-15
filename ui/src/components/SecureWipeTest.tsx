import { useState } from 'react';
import { useSecureWipe } from '../hooks/useSecureWipe';
import type { Device, WipePlan, BackupResult } from '../types/securewipe';

export function SecureWipeTest() {
    const { discover, planWipe, backup, logs, running, clearLogs } = useSecureWipe();
    const [status, setStatus] = useState<string>('Ready');
    const [devices, setDevices] = useState<Device[]>([]);
    const [wipePlan, setWipePlan] = useState<WipePlan | null>(null);
    const [backupResult, setBackupResult] = useState<BackupResult | null>(null);

    const handleDiscover = async () => {
        try {
            setStatus('Discovering devices...');
            console.log('🔍 Starting device discovery...');
            const discoveredDevices = await discover();
            console.log('✅ Raw discovered devices result:', discoveredDevices);
            console.log('📊 Device count:', discoveredDevices.length);
            console.log('📋 Devices detail:', JSON.stringify(discoveredDevices, null, 2));
            setDevices(discoveredDevices);
            setStatus(`✅ Found ${discoveredDevices.length} devices`);
        } catch (error) {
            console.error('❌ Discovery failed:', error);
            setStatus(`❌ Discovery failed: ${error}`);
        }
    };

    const handlePlanWipe = async () => {
        try {
            setStatus('Creating wipe plan...');
            console.log('🗂️ Starting wipe plan creation...');
            const plan = await planWipe({
                device: '/dev/disk2',
                samples: 128,
                isoMode: false,
                noEnrich: false
            });
            console.log('✅ Raw wipe plan result:', plan);
            console.log('📋 Plan detail:', JSON.stringify(plan, null, 2));
            setWipePlan(plan);
            setStatus(`✅ Wipe plan created: ${plan.main_method || 'Unknown method'}`);
        } catch (error) {
            console.error('❌ Plan creation failed:', error);
            setStatus(`❌ Plan creation failed: ${error}`);
        }
    };

    const handleBackup = async () => {
        try {
            setStatus('Running backup...');
            const result = await backup({
                device: '/dev/disk2',
                dest: '~/SecureWipe/test-backup',
                sign: true,
                includePaths: ['/Users/user/Documents', '/Users/user/Pictures']
            });
            console.log('Backup result:', result);
            setBackupResult(result);
            setStatus(`✅ Backup completed`);
        } catch (error) {
            console.error('Backup failed:', error);
            setStatus(`❌ Backup failed: ${error}`);
        }
    };

    return (
        <div style={{ padding: '20px', maxWidth: '800px' }}>
            <h2>SecureWipe CLI Integration Test</h2>

            {/* Platform Detection */}
            <div style={{
                marginBottom: '20px',
                padding: '15px',
                backgroundColor: '#fff3cd',
                border: '1px solid #ffeaa7',
                borderRadius: '4px'
            }}>
                <h4>🔍 Platform Detection</h4>
                <p><strong>Current OS:</strong> {navigator.platform}</p>
                <p><strong>SecureWipe Support:</strong> {navigator.platform.includes('Linux') ? '✅ Full Support' : '⚠️ Linux Required'}</p>

                {!navigator.platform.includes('Linux') && (
                    <div style={{ marginTop: '10px', padding: '10px', backgroundColor: '#f8d7da', borderRadius: '4px' }}>
                        <strong>Note:</strong> SecureWipe is designed for Linux systems.
                        The buttons below will show platform error messages explaining how to test on Linux.
                    </div>
                )}
            </div>

            <div style={{ marginBottom: '20px' }}>
                <button onClick={handleDiscover} disabled={running} style={{ margin: '5px' }}>
                    Discover Real Devices
                </button>
                <button onClick={handlePlanWipe} disabled={running} style={{ margin: '5px' }}>
                    Plan Wipe
                </button>
                <button onClick={handleBackup} disabled={running} style={{ margin: '5px' }}>
                    Test Backup
                </button>
                <button onClick={clearLogs} style={{ margin: '5px' }}>
                    Clear Logs
                </button>
                <button
                    onClick={() => {
                        setDevices([]);
                        setWipePlan(null);
                        setBackupResult(null);
                        setStatus('Ready');
                    }}
                    style={{ margin: '5px', backgroundColor: '#ffeaa7' }}
                >
                    Clear Results
                </button>
            </div>

            <div>
                <h3>Status: {running ? 'Running...' : 'Idle'}</h3>
                <div style={{
                    padding: '10px',
                    marginBottom: '10px',
                    backgroundColor: '#f0f8ff',
                    border: '1px solid #ddd',
                    borderRadius: '4px'
                }}>
                    <strong>Current Operation:</strong> {status}
                </div>

                {/* Discovered Devices */}
                {devices.length > 0 && (
                    <div style={{ marginBottom: '20px' }}>
                        <h4>📱 Discovered Devices ({devices.length}):</h4>
                        <div style={{
                            maxHeight: '200px',
                            overflowY: 'auto',
                            border: '1px solid #ddd',
                            padding: '10px',
                            backgroundColor: '#f9f9f9',
                            borderRadius: '4px'
                        }}>
                            {devices.map((device, index) => (
                                <div key={index} style={{
                                    marginBottom: '10px',
                                    padding: '8px',
                                    backgroundColor: 'white',
                                    borderRadius: '4px',
                                    border: '1px solid #eee'
                                }}>
                                    <div style={{ fontWeight: 'bold' }}>{device.name}</div>
                                    <div style={{ fontSize: '12px', color: '#666' }}>
                                        Model: {device.model || 'N/A'} |
                                        Serial: {device.serial || 'N/A'} |
                                        Size: {Math.round(device.capacity_bytes / (1024 ** 3))}GB
                                    </div>
                                    <div style={{ fontSize: '12px' }}>
                                        Bus: {device.bus || 'N/A'} |
                                        Risk: <span style={{
                                            color: device.risk_level === 'CRITICAL' ? 'red' :
                                                device.risk_level === 'HIGH' ? 'orange' : 'green',
                                            fontWeight: 'bold'
                                        }}>{device.risk_level}</span>
                                    </div>
                                    <div style={{ fontSize: '11px', color: '#888' }}>
                                        Mounts: {device.mountpoints?.join(', ') || 'None'}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Wipe Plan */}
                {wipePlan && (
                    <div style={{ marginBottom: '20px' }}>
                        <h4>🗂️ Wipe Plan:</h4>
                        <div style={{
                            padding: '10px',
                            backgroundColor: '#f0f8ff',
                            border: '1px solid #ddd',
                            borderRadius: '4px',
                            fontSize: '12px'
                        }}>
                            <div><strong>Device:</strong> {wipePlan.device_name}</div>
                            <div><strong>Method:</strong> {wipePlan.main_method}</div>
                            <div><strong>Steps:</strong> {wipePlan.steps?.length || 0}</div>
                        </div>
                    </div>
                )}

                {/* Backup Result */}
                {backupResult && (
                    <div style={{ marginBottom: '20px' }}>
                        <h4>💾 Backup Result:</h4>
                        <div style={{
                            padding: '10px',
                            backgroundColor: '#f0f8ff',
                            border: '1px solid #ddd',
                            borderRadius: '4px',
                            fontSize: '12px'
                        }}>
                            <div><strong>Backup Path:</strong> {backupResult.backup_path}</div>
                            <div><strong>Manifest Path:</strong> {backupResult.manifest_path}</div>
                            <div><strong>Integrity Checks:</strong> {backupResult.integrity_checks}</div>
                        </div>
                    </div>
                )}

                <h4>Live Logs ({logs.length} entries):</h4>
                <div
                    style={{
                        border: '1px solid #ccc',
                        padding: '10px',
                        height: '300px',
                        overflowY: 'auto',
                        fontFamily: 'monospace',
                        fontSize: '12px',
                        backgroundColor: '#f5f5f5'
                    }}
                >
                    {logs.map((log, index) => (
                        <div
                            key={index}
                            style={{
                                color: log.stream === 'stderr' ? 'red' : 'black',
                                marginBottom: '2px'
                            }}
                        >
                            <span style={{ color: '#666' }}>[{new Date(parseInt(log.ts)).toLocaleTimeString()}]</span>
                            <span style={{ color: '#999' }}> [{log.stream}]</span> {log.line}
                        </div>
                    ))}
                    {logs.length === 0 && (
                        <div style={{ color: '#999' }}>No logs yet. Click a button to test the CLI integration.</div>
                    )}
                </div>
            </div>

            <div style={{ marginTop: '20px', fontSize: '14px', color: '#666' }}>
                <h4>📋 Testing Instructions:</h4>
                <div style={{ marginBottom: '15px' }}>
                    <h5>🐧 For Linux Users (Real Device Discovery):</h5>
                    <ol>
                        <li>Build CLI: <code>cd core && cargo build --release</code></li>
                        <li>Add to PATH: <code>export PATH="$PWD/target/release:$PATH"</code></li>
                        <li>Verify: <code>securewipe --help</code></li>
                        <li>Test UI: Click buttons above to see real devices</li>
                    </ol>
                </div>

                <div style={{ marginBottom: '15px' }}>
                    <h5>🍎 For macOS/Windows Users:</h5>
                    <ul>
                        <li>SecureWipe requires Linux tools (lsblk, hdparm, nvme-cli)</li>
                        <li>Buttons above will show helpful error messages</li>
                        <li>Use Linux VM or WSL for testing real device discovery</li>
                    </ul>
                </div>

                <div>
                    <h5>🔐 Security Features:</h5>
                    <ul>
                        <li><strong>Whitelist:</strong> Only discover/backup/cert/wipe-planning allowed</li>
                        <li><strong>No Destructive Wipe:</strong> Execution flags are blocked</li>
                        <li><strong>Real-time Logs:</strong> Live stdout/stderr streaming</li>
                        <li><strong>Platform Detection:</strong> Automatic error handling</li>
                    </ul>
                </div>
            </div>
        </div>
    );
}