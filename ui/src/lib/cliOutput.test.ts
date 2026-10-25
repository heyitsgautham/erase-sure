import { describe, expect, it } from 'vitest';
import { buildWipePlanArgs, parseJsonOutput, toDevice, toWipePlan } from './cliOutput';
import type { CliDevice } from './cliOutput';

// Real `securewipe discover --format json` output from an older build that
// still mixed JSON log lines into stdout before the pretty-printed result.
const discoverStdout = [
    '{"level":"info","message":"Starting device discovery","timestamp":"2025-09-16T16:52:10.173781055+00:00"}',
    '{"level":"info","message":"Found 2 devices","timestamp":"2025-09-16T16:52:10.209419531+00:00"}',
    '[',
    '  {',
    '    "name": "/dev/zram0",',
    '    "model": null,',
    '    "serial": null,',
    '    "capacity_bytes": 4294967296,',
    '    "bus": null,',
    '    "mountpoints": ["[SWAP]"],',
    '    "risk_level": "HIGH"',
    '  },',
    '  {',
    '    "name": "/dev/nvme0n1",',
    '    "model": "WD PC SN740 SDDQMQD-512G-1001",',
    '    "serial": "23095R802486",',
    '    "capacity_bytes": 512110190592,',
    '    "bus": "NVMe",',
    '    "mountpoints": ["/", "/boot/efi"],',
    '    "risk_level": "CRITICAL"',
    '  }',
    ']',
];

describe('parseJsonOutput', () => {
    it('skips log lines and joins pretty-printed JSON', () => {
        const devices = parseJsonOutput<CliDevice[]>(discoverStdout);
        expect(devices).toHaveLength(2);
        expect(devices[1].name).toBe('/dev/nvme0n1');
    });

    it('parses a single-line result', () => {
        expect(parseJsonOutput<{ ok: boolean }>(['{"ok":true}'])).toEqual({ ok: true });
    });

    it('falls back to the last line that parses on its own', () => {
        const lines = ['Backup completed successfully!', '{"backup_id":"abc"}'];
        expect(parseJsonOutput<{ backup_id: string }>(lines)).toEqual({ backup_id: 'abc' });
    });

    it('rejects output that holds only log lines or blanks', () => {
        const logsOnly = ['{"level":"info","message":"hi","timestamp":"t"}', '   ', ''];
        expect(() => parseJsonOutput(logsOnly)).toThrow('No JSON output found');
    });

    it('rejects output with no JSON at all', () => {
        expect(() => parseJsonOutput(['not', 'json'])).toThrow('Failed to parse JSON output');
    });
});

describe('toDevice', () => {
    it('blocks CRITICAL devices and explains why', () => {
        const [, systemDisk] = parseJsonOutput<CliDevice[]>(discoverStdout);
        const device = toDevice(systemDisk);
        expect(device).toMatchObject({
            path: '/dev/nvme0n1',
            capacity: 512110190592,
            risk_level: 'CRITICAL',
            blocked: true,
            block_reason: 'System disk with active mount points',
        });
    });

    it('fills placeholders for missing fields and leaves other devices unblocked', () => {
        const [swap] = parseJsonOutput<CliDevice[]>(discoverStdout);
        const device = toDevice(swap);
        expect(device).toMatchObject({
            path: '/dev/zram0',
            model: 'Unknown Device',
            serial: 'N/A',
            bus: 'Unknown',
            risk_level: 'HIGH',
            blocked: false,
        });
        expect(device.block_reason).toBeUndefined();
    });

    it('defaults an unknown risk level to SAFE', () => {
        expect(toDevice({ path: '/dev/sdz' })).toMatchObject({ path: '/dev/sdz', risk_level: 'SAFE', capacity: 0 });
    });
});

describe('toWipePlan', () => {
    it('maps the CLI plan, including the reason a plan is blocked', () => {
        const plan = toWipePlan(
            {
                device: '/dev/sda',
                policy: 'CLEAR',
                main_method: 'overwrite',
                hpa_dco_clear: true,
                verification: { samples: 32 },
                blocked: true,
                reason: 'CRITICAL disk wipe blocked unless running from bootable ISO mode',
            },
            '/dev/other',
        );
        expect(plan).toEqual({
            device_path: '/dev/sda',
            policy: 'CLEAR',
            main_method: 'overwrite',
            hpa_dco_clear: true,
            verification: { samples: 32 },
            blocked: true,
            block_reason: 'CRITICAL disk wipe blocked unless running from bootable ISO mode',
        });
    });

    it('falls back to the requested device and samples', () => {
        const plan = toWipePlan({ reason: null }, '/dev/sdb', 64);
        expect(plan.device_path).toBe('/dev/sdb');
        expect(plan.policy).toBe('PURGE');
        expect(plan.verification.samples).toBe(64);
        expect(plan.blocked).toBe(false);
        expect(plan.block_reason).toBeUndefined();
        expect(toWipePlan({}, '/dev/sdb').verification.samples).toBe(128);
    });
});

describe('buildWipePlanArgs', () => {
    it('always asks for JSON so the CLI stays in planning mode', () => {
        expect(buildWipePlanArgs({ device: '/dev/sdb' })).toEqual(['wipe', '--device', '/dev/sdb', '--format', 'json']);
    });

    it('adds samples, ISO mode and the no-enrich switch when asked', () => {
        expect(buildWipePlanArgs({ device: '/dev/sdb', samples: 64, isoMode: true, noEnrich: true })).toEqual([
            'wipe',
            '--device',
            '/dev/sdb',
            '--format',
            'json',
            '--samples',
            '64',
            '--iso-mode',
            '--no-enrich',
        ]);
    });
});
