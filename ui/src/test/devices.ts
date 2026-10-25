import type { Device } from '../contexts/AppContext';

export const safeUsb: Device = {
    path: '/dev/sdb',
    model: 'SanDisk Ultra USB',
    serial: '4C530001230512',
    capacity: 32010928128,
    bus: 'usb',
    mountpoints: [],
    risk_level: 'SAFE',
    blocked: false,
};

export const systemNvme: Device = {
    path: '/dev/nvme0n1',
    model: 'Samsung SSD 980',
    serial: 'S64DNX0R123456',
    capacity: 500107862016,
    bus: 'nvme',
    mountpoints: ['/', '/boot'],
    risk_level: 'CRITICAL',
    blocked: true,
    block_reason: 'System disk with active mount points',
};
