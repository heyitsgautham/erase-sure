import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import DeviceDiscovery from '../components/discovery/DeviceDiscovery';

function WipePlanDiscover() {
    const navigate = useNavigate();
    const { state, dispatch } = useApp();

    const handleContinueToWipePlan = () => {
        if (state.selectedDevice) {
            dispatch({ type: 'SET_OPERATION_TYPE', payload: 'wipe' });
            navigate('/wipe-plan');
        }
    };

    return (
        <DeviceDiscovery
            title="📋 Wipe Plan Device Discovery"
            description="Scan and analyze storage devices for wipe planning operations. Select a device to continue with wipe strategy analysis."
            criticalSelectNote="System disk - Planning blocked, backup only"
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
                <button
                    className="btn btn-secondary"
                    onClick={handleContinueToWipePlan}
                >
                    📋 View Wipe Plan
                    <div style={{ fontSize: '0.75rem', opacity: 0.7, marginTop: '0.25rem' }}>
                        Preview wiping strategy (safe mode)
                    </div>
                </button>
            }
            selectedCriticalNote={
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
            }
        />
    );
}

export default WipePlanDiscover;
