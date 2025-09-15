import { useState } from 'react';
import { useSecureWipe } from '../hooks/useSecureWipe';

export function SecureWipeTest() {
    const { discover, planWipe, backup, logs, running, clearLogs } = useSecureWipe();
    const [status, setStatus] = useState<string>('Ready');

    const handleDiscover = async () => {
        try {
            setStatus('Discovering devices...');
            const devices = await discover();
            console.log('Discovered devices:', devices);
            setStatus(`✅ Found ${devices.length} devices`);
        } catch (error) {
            console.error('Discovery failed:', error);
            setStatus(`❌ Discovery failed: ${error}`);
        }
    };

    const handlePlanWipe = async () => {
        try {
            setStatus('Creating wipe plan...');
            const plan = await planWipe({
                device: '/dev/disk2',
                samples: 128,
                isoMode: false,
                noEnrich: false
            });
            console.log('Wipe plan:', plan);
            setStatus(`✅ Wipe plan created: ${plan.main_method}`);
        } catch (error) {
            console.error('Plan creation failed:', error);
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