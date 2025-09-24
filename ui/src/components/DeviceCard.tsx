import type { Device } from '../contexts/AppContext';

interface RiskBadgeProps {
    level: Device['risk_level'];
}

function RiskBadge({ level }: RiskBadgeProps) {
    return (
        <span className={`risk-badge ${level.toLowerCase()}`}>
            {level}
        </span>
    );
}

interface DeviceCardProps {
    device: Device;
    selected?: boolean;
    onSelect?: (device: Device) => void;
    onBlockedClick?: (device: Device) => void;
}

function DeviceCard({ device, selected = false, onSelect }: DeviceCardProps) {
    const handleClick = () => {
        // Allow selection of all devices now (critical devices can be used for backup)
        if (onSelect) {
            onSelect(device);
        }
    };

    const formatCapacity = (bytes: number) => {
        const gb = (bytes / (1024 ** 3)).toFixed(1);
        return `${gb} GB`;
    };

    const formatSerial = (serial: string) => {
        if (!serial || serial === 'N/A') return serial;
        
        // If serial is longer than 20 characters, truncate and add tooltip
        if (serial.length > 20) {
            return serial.substring(0, 20) + '...';
        }
        return serial;
    };

    const cardClassName = [
        'card',
        'device-card',
        selected ? 'selected' : '',
        device.risk_level === 'CRITICAL' ? 'critical' : ''
    ].filter(Boolean).join(' ');

    return (
        <div className={cardClassName} onClick={handleClick}>
            <div className="flex justify-between items-center mb-4">
                <h3 className="font-semibold">{device.model}</h3>
                <RiskBadge level={device.risk_level} />
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                    <span className="font-medium">Serial:</span>
                    <div 
                        title={device.serial} 
                        style={{ 
                            wordBreak: 'break-all',
                            fontSize: '0.75rem',
                            lineHeight: '1.2'
                        }}
                    >
                        {formatSerial(device.serial)}
                    </div>
                </div>
                <div>
                    <span className="font-medium">Capacity:</span>
                    <div>{formatCapacity(device.capacity)}</div>
                </div>
                <div>
                    <span className="font-medium">Bus:</span>
                    <div>{device.bus.toUpperCase()}</div>
                </div>
                <div>
                    <span className="font-medium">Path:</span>
                    <div className="text-xs">{device.path}</div>
                </div>
            </div>

            {device.mountpoints.length > 0 && (
                <div className="mt-4">
                    <span className="font-medium text-sm">Mount Points:</span>
                    <div className="text-xs">
                        {device.mountpoints.join(', ')}
                    </div>
                </div>
            )}

            {device.risk_level === 'CRITICAL' && (
                <div className="alert alert-warning mt-4">
                    <small>System disk with active mount points</small>
                </div>
            )}
        </div>
    );
}

export default DeviceCard;
