import type { ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { useApp } from '../../contexts/AppContext';
import type { Device } from '../../contexts/AppContext';
import { useSecureWipe } from '../../hooks/useSecureWipe';
import { createLogger } from '../../lib/logger';
import DeviceCard from '../DeviceCard';

const log = createLogger('discovery');

interface DeviceDiscoveryProps {
    title: string;
    description: string;
    /** Show "Back to Home", which also clears the selection. */
    showBackButton?: boolean;
    /** Added to the selection toast when the user picks a system disk. */
    criticalSelectNote: string;
    /** The banner shown when the scan found system disks. */
    criticalWarning: (criticalCount: number) => ReactNode;
    /** The buttons for what to do with the selected device. */
    actions: ReactNode;
    /** Extra guidance under the buttons when the selected device is a system disk. */
    selectedCriticalNote?: ReactNode;
}

/**
 * Scan for devices, list them and let the user pick one.
 *
 * Each discovery screen supplies its own wording, warnings and next-step
 * buttons; the scanning, listing and selection behave the same everywhere.
 */
function DeviceDiscovery({
    title,
    description,
    showBackButton = true,
    criticalSelectNote,
    criticalWarning,
    actions,
    selectedCriticalNote,
}: DeviceDiscoveryProps) {
    const navigate = useNavigate();
    const { state, dispatch, addToast } = useApp();
    const { discover, running } = useSecureWipe();

    const handleScanDevices = async () => {
        try {
            await discover();
        } catch (error) {
            // discover() already told the user.
            log.error('device discovery failed', undefined, error);
        }
    };

    const handleDeviceSelect = (device: Device) => {
        dispatch({ type: 'SELECT_DEVICE', payload: device });

        if (device.risk_level === 'CRITICAL') {
            addToast(`Selected ${device.model} (${criticalSelectNote})`, 'warning');
        } else if (device.risk_level === 'HIGH') {
            addToast(`Selected ${device.model} (High risk device - proceed with caution)`, 'warning');
        } else {
            addToast(`Selected ${device.model} (Safe device selected)`, 'success');
        }
    };

    const handleBackToHome = () => {
        dispatch({ type: 'SELECT_DEVICE', payload: null });
        navigate('/');
    };

    const criticalDeviceCount = state.devices.filter(d => d.risk_level === 'CRITICAL').length;
    const safeDeviceCount = state.devices.filter(d => d.risk_level === 'SAFE').length;

    return (
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
            <div className="mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    {title}
                </h2>
                <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                    {description}
                </p>

                <div className="flex gap-4 items-center mb-6">
                    <button
                        className="btn btn-primary"
                        onClick={handleScanDevices}
                        disabled={running}
                    >
                        {running ? (
                            <span className="loading-text">
                                🔄 <span className="loading-dots">Scanning</span>
                            </span>
                        ) : (
                            '🔍 Scan Devices'
                        )}
                    </button>

                    {showBackButton && (
                        <button
                            className="btn btn-secondary"
                            onClick={handleBackToHome}
                        >
                            ← Back to Home
                        </button>
                    )}

                    {state.devices.length > 0 && (
                        <div className="text-sm" style={{ color: '#64748b' }}>
                            Found {state.devices.length} device(s): {safeDeviceCount} safe, {criticalDeviceCount} critical
                        </div>
                    )}
                </div>
            </div>

            {criticalDeviceCount > 0 && criticalWarning(criticalDeviceCount)}

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

                    {state.selectedDevice && (
                        <div className="card">
                            <h3 className="font-semibold mb-4">Selected Device: {state.selectedDevice.model}</h3>

                            <div className="flex gap-4">
                                {actions}
                            </div>

                            {state.selectedDevice.risk_level === 'CRITICAL' && selectedCriticalNote}
                        </div>
                    )}
                </>
            ) : (
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
            )}

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

export default DeviceDiscovery;
