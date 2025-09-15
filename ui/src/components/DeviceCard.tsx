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

function DeviceCard({ device, selected = false, onSelect, onBlockedClick }: DeviceCardProps) {
    const isBlocked = device.risk_level === 'CRITICAL';
    
    const handleClick = () => {
        if (isBlocked && onBlockedClick) {
            onBlockedClick(device);
        } else if (!isBlocked && onSelect) {
            onSelect(device);
        }
    };

    const formatCapacity = (bytes: number) => {
        const gb = (bytes / (1024 ** 3)).toFixed(1);
        return `${gb} GB`;
    };

    const cardClassName = [
        'card',
        'device-card',
        selected ? 'selected' : '',
        isBlocked ? 'blocked' : ''
    ].filter(Boolean).join(' ');

    return (
        <div className={cardClassName} onClick={handleClick}>
            <div className="flex justify-between items-center mb-4">
                <h3 className="font-semibold">{device.model || device.name}</h3>
                <RiskBadge level={device.risk_level} />
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                    <span className="font-medium">Serial:</span>
                    <div>{device.serial || 'N/A'}</div>
                </div>
                <div>
                    <span className="font-medium">Capacity:</span>
                    <div>{formatCapacity(device.capacity_bytes)}</div>
                </div>
                <div>
                    <span className="font-medium">Bus:</span>
                    <div>{device.bus?.toUpperCase() || 'N/A'}</div>
                </div>
                <div>
                    <span className="font-medium">Path:</span>
                    <div className="text-xs">{device.name}</div>
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

            {isBlocked && (
                <div className="alert alert-error mt-4">
                    <small>Critical system device - blocked for safety</small>
                </div>
            )}
        </div>
    );
}

export default DeviceCard;
