import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe } from '../hooks/useSecureWipe';
import DeviceCard from '../components/DeviceCard';
import type { Device } from '../contexts/AppContext';

function WipePlanDiscover() {
    const navigate = useNavigate();
    const { state, dispatch } = useApp();
    const { discover, running } = useSecureWipe();

    const handleScanDevices = async () => {
        try {
            await discover();
        } catch (error) {
            console.error('Device discovery failed:', error);
        }
    };

    const handleDeviceSelect = (device: Device) => {
        // Allow selection of critical devices for planning operations only
        dispatch({ type: 'SELECT_DEVICE', payload: device });

        // Show appropriate toast based on device risk level
        let message = `Selected ${device.model}`;
        let type: 'success' | 'warning' | 'info' = 'success';

        if (device.risk_level === 'CRITICAL') {
            message += ' (System disk - Planning blocked, backup only)';
            type = 'warning';
        } else if (device.risk_level === 'HIGH') {
            message += ' (High risk device - proceed with caution)';
            type = 'warning';
        } else {
            message += ' (Safe device selected)';
            type = 'success';
        }

        dispatch({
            type: 'ADD_TOAST',
            payload: {
                id: Date.now().toString(),
                type,
                message
            }
        });
    };

    const handleContinueToWipePlan = () => {
        if (state.selectedDevice) {
            dispatch({ type: 'SET_OPERATION_TYPE', payload: 'wipe' });
            navigate('/wipe-plan');
        }
    };

    const handleBackToHome = () => {
        dispatch({ type: 'SELECT_DEVICE', payload: null }); // Clear selection when going back
        navigate('/');
    };

    // Note: Removed auto-scan to prevent duplicate toast messages
    // Users can manually click "Scan Devices" when needed

    const criticalDeviceCount = state.devices.filter(d => d.risk_level === 'CRITICAL').length;
    const safeDeviceCount = state.devices.filter(d => d.risk_level === 'SAFE').length;

    return (
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
            <div className="mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    📋 Wipe Plan Device Discovery
                </h2>
                <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                    Scan and analyze storage devices for wipe planning operations.
                    Select a device to continue with wipe strategy analysis.
                </p>

                <div className="flex gap-4 items-center mb-6">
                    <button
                        className="btn btn-primary"
                        onClick={handleScanDevices}
                        disabled={state.isLoading || running}
                    >
                        {(state.isLoading || running) ? (
                            <span className="loading-text">
                                🔄 <span className="loading-dots">Scanning</span>
                            </span>
                        ) : (
                            '🔍 Scan Devices'
                        )}
                    </button>

                    <button
                        className="btn btn-secondary"
                        onClick={handleBackToHome}
                    >
                        ← Back to Home
                    </button>

                    {state.devices.length > 0 && (
                        <div className="text-sm" style={{ color: '#64748b' }}>
                            Found {state.devices.length} device(s): {safeDeviceCount} safe, {criticalDeviceCount} critical
                        </div>
                    )}
                </div>
            </div>

            {/* Critical Device Warning */}
            {criticalDeviceCount > 0 && (
                <div className="alert alert-warning mb-6">
                    <h4 className="font-semibold mb-2">⚠️ Critical Devices Detected</h4>
                    <p className="text-sm">
                        {criticalDeviceCount} device(s) are marked as CRITICAL (system disks or mounted volumes).
                        These devices are blocked from wipe operations unless running from ISO mode.
                        Consider backing up important data first.
                    </p>
                </div>
            )}

            {/* Device List */}
            {state.devices.length > 0 ? (
                <>
                    <div className="grid grid-cols-1 gap-4 mb-6">
                        {state.devices.map((device) => (
                            <DeviceCard
                                key={device.path}
                                device={device}
                                selected={state.selectedDevice?.path === device.path}
                                onSelect={handleDeviceSelect}
                            />
                        ))}
                    </div>

                    {/* Action Buttons */}
                    {state.selectedDevice && (
                        <div className="card">
                            <h3 className="font-semibold mb-4">Selected Device: {state.selectedDevice.model}</h3>

                            <div className="flex gap-4">
                                <button
                                    className="btn btn-secondary"
                                    onClick={handleContinueToWipePlan}
                                >
                                    📋 View Wipe Plan
                                    <div style={{ fontSize: '0.75rem', opacity: 0.7, marginTop: '0.25rem' }}>
                                        Preview wiping strategy (safe mode)
                                    </div>
                                </button>
                            </div>

                            {state.selectedDevice.risk_level === 'CRITICAL' && (
                                <div className="alert alert-warning mt-4">
                                    <h4 className="font-semibold mb-2">⚠️ System Disk Selected</h4>
                                    <div className="text-sm space-y-2">
                                        <p>
                                            <strong>Wipe Plan:</strong> ✅ Allowed - Preview mode only (no actual wiping)
                                        </p>
                                        <p>
                                            <strong>Destructive Wipe:</strong> ❌ Blocked - Boot from SecureWipe ISO to enable wiping of system disks
                                        </p>
                                        <p>
                                            <strong>Backup:</strong> ✅ Allowed - Will backup user files only (Documents, Pictures, etc.)
                                        </p>
                                    </div>
                                </div>
                            )}
                        </div>
                    )}
                </>
            ) : (
                !state.isLoading && (
                    <div className="card text-center" style={{ padding: '3rem' }}>
                        <div style={{ fontSize: '4rem', marginBottom: '1rem', opacity: 0.3 }}>💾</div>
                        <h3 className="font-semibold mb-2">No Devices Found</h3>
                        <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                            Click "Scan Devices" to discover available storage devices.
                        </p>
                        <button
                            className="btn btn-primary"
                            onClick={handleScanDevices}
                        >
                            🔍 Start Device Scan
                        </button>
                    </div>
                )
            )}

            {/* Current Operation Status */}
            {state.currentOperation && (
                <div className="alert alert-info mt-6">
                    <div className="flex items-center gap-2">
                        <div className="progress-bar" style={{ width: '20px', height: '20px', borderRadius: '50%' }}>
                            <div className="progress-fill" style={{ animation: 'spin 1s linear infinite', borderRadius: '50%' }}></div>
                        </div>
                        <span>{state.currentOperation}</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default WipePlanDiscover;