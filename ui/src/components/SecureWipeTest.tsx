import { useSecureWipe } from '../hooks/useSecureWipe';

export function SecureWipeTest() {
    const { discover, planWipe, backup, logs, running, clearLogs } = useSecureWipe();

    const handleDiscover = async () => {
        try {
            const devices = await discover();
            console.log('Discovered devices:', devices);
        } catch (error) {
            console.error('Discovery failed:', error);
        }
    };

    const handlePlanWipe = async () => {
        try {
            const plan = await planWipe({
                device: '/dev/sdb',
                samples: 128,
                isoMode: false,
                noEnrich: false
            });
            console.log('Wipe plan:', plan);
        } catch (error) {
            console.error('Plan creation failed:', error);
        }
    };

    const handleBackup = async () => {
        try {
            const result = await backup({
                device: '/dev/sdb',
                dest: '~/SecureWipe/test-backup',
                sign: true,
                includePaths: ['/home/user/Documents', '/home/user/Pictures']
            });
            console.log('Backup result:', result);
        } catch (error) {
            console.error('Backup failed:', error);
        }
    };

    return (
        <div style={{ padding: '20px', maxWidth: '800px' }}>
            <h2>SecureWipe CLI Integration Test</h2>

            <div style={{ marginBottom: '20px' }}>
                <button onClick={handleDiscover} disabled={running} style={{ margin: '5px' }}>
                    Discover Devices
                </button>
                <button onClick={handlePlanWipe} disabled={running} style={{ margin: '5px' }}>
                    Plan Wipe (Mock Device)
                </button>
                <button onClick={handleBackup} disabled={running} style={{ margin: '5px' }}>
                    Test Backup (Mock Device)
                </button>
                <button onClick={clearLogs} style={{ margin: '5px' }}>
                    Clear Logs
                </button>
            </div>

            <div>
                <h3>Status: {running ? 'Running...' : 'Idle'}</h3>

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
                <h4>Expected Behavior:</h4>
                <ul>
                    <li><strong>Discover:</strong> Should call `securewipe discover --format json` and show real device data</li>
                    <li><strong>Plan Wipe:</strong> Should call `securewipe wipe --device /dev/sdb --format json --samples 128` (planning only)</li>
                    <li><strong>Backup:</strong> Should call `securewipe backup --device /dev/sdb --dest ~/SecureWipe/test-backup --sign --paths ...`</li>
                    <li><strong>Security:</strong> Destructive flags like --apply, --execute, --force are blocked by the backend</li>
                    <li><strong>Logs:</strong> Should stream stdout/stderr in real-time with timestamps</li>
                </ul>
            </div>
        </div>
    );
}