import { describe, expect, it } from 'vitest';
import { CertificateSummarySchema, CliDeviceSchema, VerifyResultSchema, WipeDeviceInfoSchema } from './schemas';

describe('CertificateSummarySchema', () => {
    const cert = {
        cert_id: 'abc',
        cert_type: 'backup',
        created_at: '2026-01-01T00:00:00Z',
        signature: { alg: 'Ed25519' },
    };

    it('accepts a certificate and keeps its other fields', () => {
        expect(CertificateSummarySchema.parse(cert)).toEqual(cert);
    });

    it.each([
        ['an unknown type', { ...cert, cert_type: 'restore' }],
        ['an empty id', { ...cert, cert_id: '' }],
        ['a date that does not parse', { ...cert, created_at: 'yesterday' }],
        ['a missing id', { cert_type: 'wipe', created_at: cert.created_at }],
    ])('rejects %s', (_, value) => {
        expect(CertificateSummarySchema.safeParse(value).success).toBe(false);
    });
});

describe('CliDeviceSchema', () => {
    it('needs a name or a path', () => {
        expect(CliDeviceSchema.safeParse({ path: '/dev/sdb' }).success).toBe(true);
        expect(CliDeviceSchema.safeParse({ model: 'USB' }).success).toBe(false);
    });

    it('rejects a risk level the UI cannot show', () => {
        expect(CliDeviceSchema.safeParse({ name: '/dev/sdb', risk_level: 'UNKNOWN' }).success).toBe(false);
    });
});

describe('VerifyResultSchema', () => {
    it('accepts the portal response and rejects a missing errors list', () => {
        const result = {
            schema_valid: true,
            signature_valid: null,
            hash_valid: true,
            chain_valid: null,
            cert_summary: { cert_id: 'abc', cert_type: 'backup', device_model: 'USB' },
            computed: { sha256: 'ff', note: null },
            errors: [],
        };

        expect(VerifyResultSchema.parse(result)).toEqual(result);
        expect(VerifyResultSchema.safeParse({ ...result, errors: undefined }).success).toBe(false);
    });
});

describe('WipeDeviceInfoSchema', () => {
    it('accepts lsblk output with the criticality flag', () => {
        const info = {
            blockdevices: [{ name: 'sdb', serial: null, size: '32G', is_critical: false, path: '/dev/sdb' }],
        };
        expect(WipeDeviceInfoSchema.parse(info)).toEqual(info);
    });
});
