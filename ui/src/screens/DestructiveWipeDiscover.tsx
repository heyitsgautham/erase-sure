import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import DeviceDiscovery from '../components/discovery/DeviceDiscovery';

function DestructiveWipeDiscover() {
    const navigate = useNavigate();
    const { state, dispatch, addToast } = useApp();

    const handleContinueToDestructiveWipe = () => {
        if (!state.selectedDevice) return;
        if (state.selectedDevice.risk_level === 'CRITICAL') {
            addToast('Cannot wipe CRITICAL devices unless running from ISO mode', 'error');
            return;
        }
        dispatch({ type: 'SET_OPERATION_TYPE', payload: 'wipe' });
        navigate('/destructive-wipe');
    };

    return (
        <DeviceDiscovery
            title="🗑️ Destructive Wipe Device Discovery"
            description="Scan and analyze storage devices for destructive wipe operations. Select a device to continue with real disk wiping."
            criticalSelectNote="System disk - Wiping blocked for safety"
            criticalWarning={(count) => (
                <div className="alert alert-error mb-6">
                    <h4 className="font-semibold mb-2">🚫 Critical Devices Detected</h4>
                    <p className="text-sm">
                        {count} device(s) are marked as CRITICAL (system disks or mounted volumes). These devices CANNOT
                        be wiped in normal operation mode due to active mount points and system usage.
                    </p>
                    <p className="text-sm mt-2">
                        <strong>Boot from SecureWipe ISO</strong> to enable wiping of system disks and mounted volumes.
                    </p>
                </div>
            )}
            actions={
                <button
                    className="btn btn-danger"
                    onClick={handleContinueToDestructiveWipe}
                    style={{
                        backgroundColor: '#dc2626',
                        color: 'white',
                        fontWeight: 'bold',
                    }}
                >
                    🗑️ Start Destructive Wipe
                    <div style={{ fontSize: '0.75rem', opacity: 0.9, marginTop: '0.25rem' }}>
                        Real disk wiping with NIST-aligned deletion
                    </div>
                </button>
            }
            selectedCriticalNote={
                <div className="alert alert-error mt-4">
                    <h4 className="font-semibold mb-2">🚫 System Disk Selected</h4>
                    <div className="text-sm space-y-2">
                        <p>
                            <strong>Destructive Wipe:</strong> ❌ Blocked - Boot from SecureWipe ISO to enable wiping of
                            system disks
                        </p>
                        <p>
                            <strong>Backup:</strong> ✅ Allowed - Will backup user files only (Documents, Pictures,
                            etc.)
                        </p>
                    </div>
                </div>
            }
        />
    );
}

export default DestructiveWipeDiscover;
