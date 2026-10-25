import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import DeviceDiscovery from '../components/discovery/DeviceDiscovery';

function BackupDiscover() {
    const navigate = useNavigate();
    const { state, dispatch } = useApp();

    const handleContinueToBackup = () => {
        if (state.selectedDevice) {
            dispatch({ type: 'SET_OPERATION_TYPE', payload: 'backup' });
            navigate('/backup');
        }
    };

    return (
        <DeviceDiscovery
            title="📦 Backup Device Discovery"
            description="Scan and analyze storage devices for secure backup operations. Select a device to continue with encrypted backup."
            criticalSelectNote="System disk - Backup allowed"
            criticalWarning={(count) => (
                <div className="alert alert-warning mb-6">
                    <h4 className="font-semibold mb-2">⚠️ Critical Devices Detected</h4>
                    <p className="text-sm">
                        {count} device(s) are marked as CRITICAL (system disks). Backup operations are allowed on system
                        disks to preserve user data.
                    </p>
                </div>
            )}
            actions={
                <button className="btn btn-primary" onClick={handleContinueToBackup}>
                    📦 Continue to Backup
                    <div style={{ fontSize: '0.75rem', opacity: 0.8, marginTop: '0.25rem' }}>
                        Encrypt and backup device data
                    </div>
                </button>
            }
        />
    );
}

export default BackupDiscover;
