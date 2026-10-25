import type { Device } from '../../contexts/AppContext';

/** Model, capacity and serial of the device being backed up. */
function SourceDeviceCard({ device }: { device: Device }) {
    return (
        <div className="card mb-8" style={{ 
            background: 'linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%)', 
            border: '1px solid #e2e8f0',
            borderRadius: '16px',
            padding: '2rem'
        }}>
            <div className="flex items-center gap-4 mb-6">
                <div style={{ 
                    width: '48px', 
                    height: '48px', 
                    background: device.risk_level === 'CRITICAL' 
                        ? 'linear-gradient(135deg, #fee2e2 0%, #fecaca 100%)' 
                        : 'linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%)',
                    borderRadius: '12px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '24px'
                }}>
                    💾
                </div>
                <div className="flex-1">
                    <h2 className="font-semibold mb-2" style={{ fontSize: '1.5rem', color: '#1e293b' }}>
                        Source Device
                    </h2>
                    <p style={{ color: '#64748b', fontSize: '1rem' }}>
                        The device that will be backed up before wiping
                    </p>
                </div>
                <div className={`risk-badge ${device.risk_level.toLowerCase()}`} style={{
                    fontSize: '0.875rem',
                    fontWeight: '600',
                    padding: '0.5rem 1rem',
                    borderRadius: '8px'
                }}>
                    {device.risk_level}
                </div>
            </div>
            
            <div className="grid grid-cols-3 gap-6">
                <div className="text-center p-6 bg-white rounded-xl border border-gray-100" style={{ boxShadow: '0 1px 3px 0 rgb(0 0 0 / 0.1)' }}>
                    <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🏷️</div>
                    <div className="font-semibold text-gray-900" style={{ fontSize: '1.125rem', marginBottom: '0.5rem' }}>
                        {device.model}
                    </div>
                    <div className="text-sm text-gray-500">Device Model</div>
                </div>
                <div className="text-center p-6 bg-white rounded-xl border border-gray-100" style={{ boxShadow: '0 1px 3px 0 rgb(0 0 0 / 0.1)' }}>
                    <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>📊</div>
                    <div className="font-semibold text-gray-900" style={{ fontSize: '1.125rem', marginBottom: '0.5rem' }}>
                        {(device.capacity / (1024 ** 3)).toFixed(1)} GB
                    </div>
                    <div className="text-sm text-gray-500">Storage Capacity</div>
                </div>
                <div className="text-center p-6 bg-white rounded-xl border border-gray-100" style={{ boxShadow: '0 1px 3px 0 rgb(0 0 0 / 0.1)' }}>
                    <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🔢</div>
                    <div className="font-semibold text-gray-900" style={{ 
                        fontSize: '0.8rem', 
                        marginBottom: '0.5rem', 
                        fontFamily: 'monospace',
                        wordBreak: 'break-all',
                        lineHeight: '1.2'
                    }}>
                        {device.serial}
                    </div>
                    <div className="text-sm text-gray-500">Serial Number</div>
                </div>
            </div>
        </div>
    );
}

export default SourceDeviceCard;
