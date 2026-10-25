import { describe, expect, it } from 'vitest';
import { certTypeIcon, certTypeLabel, certificateDir, pdfCandidates, toCertificate, verifyUrl } from './certificates';

describe('certificates', () => {
    it('builds the folder and PDF locations from the home folder', () => {
        expect(certificateDir('/home/me')).toBe('/home/me/SecureWipe/certificates');
        expect(pdfCandidates('/home/me/SecureWipe/certificates/a.json', 'a', '/home/me')).toEqual([
            '/home/me/SecureWipe/backups/a.pdf',
            '/home/me/SecureWipe/certificates/a.pdf',
        ]);
    });

    it('swaps only the final .json extension for the PDF next to it', () => {
        const [, beside] = pdfCandidates('/data/x.json.d/cert.json', 'cert', '/home/me');
        expect(beside).toBe('/data/x.json.d/cert.pdf');
    });

    it('encodes the certificate ID in the verify link', () => {
        expect(verifyUrl('https://verify.example', 'a b/c')).toBe('https://verify.example/verify?cert_id=a%20b%2Fc');
    });

    it('describes a certificate for the list', () => {
        const cert = toCertificate(
            '/c/abc.json',
            { cert_id: 'abc', cert_type: 'wipe', created_at: '2026-03-04T05:06:07Z' },
            undefined,
            'http://localhost:8000',
        );

        expect(cert).toEqual({
            id: 'abc',
            type: 'wipe',
            filename: 'abc.json',
            path: '/c/abc.json',
            pdfPath: undefined,
            created: new Date('2026-03-04T05:06:07Z'),
            verifyUrl: 'http://localhost:8000/verify?cert_id=abc',
        });
        expect(certTypeLabel(cert.type)).toBe('Wipe Certificate');
        expect(certTypeIcon('backup')).toBe('📦');
    });
});
