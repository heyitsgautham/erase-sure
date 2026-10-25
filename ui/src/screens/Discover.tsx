import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import DeviceDiscovery from '../components/discovery/DeviceDiscovery';

function Discover() {
    const navigate = useNavigate();
    const { state, dispatch, addToast } = useApp();

    const handleContinueToWipePlan = () => {
        if (!state.selectedDevice) return;
        if (state.selectedDevice.risk_level === 'CRITICAL') {
            addToast('Wipe planning blocked for system disks. Use backup instead or boot from ISO.', 'error');
            return;
        }
        dispatch({ type: 'SET_OPERATION_TYPE', payload: 'wipe' });
        navigate('/wipe-plan');
    };

    const handleContinueToBackup = () => {
        if (state.selectedDevice) {
            dispatch({ type: 'SET_OPERATION_TYPE', payload: 'backup' });
            navigate('/backup');
        }
    };

    return (
        <DeviceDiscovery
            title="Device Discovery"
            description="Scan and analyze storage devices for secure wiping operations. Select a device to continue with backup or wipe planning."
            showBackButton={false}
            criticalSelectNote="System disk - Backup only, wiping blocked for safety"
            criticalWarning={(count) => (
                <div className="alert alert-warning mb-6">
                    <h4 className="font-semibold mb-2">⚠️ Critical Devices Detected</h4>
                    <p className="text-sm">
                        {count} device(s) are marked as CRITICAL (system disks or mounted volumes).
                        These devices are blocked from wipe operations unless running from ISO mode.
                        Consider backing up important data first.
                    </p>
                </div>
            )}
            actions={
                <>
                    <button
                        className="btn btn-primary"
                        onClick={handleContinueToBackup}
                    >
                        📦 Continue to Backup
                        <div style={{ fontSize: '0.75rem', opacity: 0.8, marginTop: '0.25rem' }}>
                            Encrypt and backup device data first
                        </div>
                    </button>

                    <button
                        className="btn btn-secondary"
                        onClick={handleContinueToWipePlan}
                    >
                        📋 View Wipe Plan
                        <div style={{ fontSize: '0.75rem', opacity: 0.7, marginTop: '0.25rem' }}>
                            Preview wiping strategy (safe mode)
                        </div>
                    </button>
                </>
            }
            selectedCriticalNote={
                <div className="alert alert-warning mt-4">
                    <h4 className="font-semibold mb-2">⚠️ System Disk Selected</h4>
                    <div className="text-sm space-y-2">
                        <p>
                            <strong>Backup:</strong> ✅ Allowed - Will backup user files only (Documents, Pictures, etc.)
                        </p>
                        <p>
                            <strong>Wipe:</strong> ❌ Blocked - Boot from SecureWipe ISO to enable wiping of system disks
                        </p>
                    </div>
                </div>
            }
        />
    );
}

export default Discover;
