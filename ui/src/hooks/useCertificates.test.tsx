import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useCertificates } from './useCertificates';
import { installFakeTauri } from '../test/fakeTauri';

const HOME = '/home/me';
const DIR = `${HOME}/SecureWipe/certificates`;

function fakeCertificates(files: Record<string, unknown>, pdfs: string[] = []) {
    const tauri = installFakeTauri();
    tauri.on('get_home_dir', () => HOME);
    tauri.on('list_cert_files', () => Object.keys(files));
    tauri.on('read_file_content', (args) => {
        const content = files[String(args.filePath)];
        return typeof content === 'string' ? content : JSON.stringify(content);
    });
    tauri.on('file_exists', (args) => pdfs.includes(String(args.filePath)));
    return tauri;
}

describe('useCertificates', () => {
    it('lists valid certificates newest first with their verify links', async () => {
        fakeCertificates({
            [`${DIR}/old.json`]: { cert_id: 'old', cert_type: 'wipe', created_at: '2026-01-01T00:00:00Z' },
            [`${DIR}/new.json`]: { cert_id: 'new', cert_type: 'backup', created_at: '2026-02-01T00:00:00Z' },
        });
        const { result } = renderHook(() => useCertificates());

        await waitFor(() => expect(result.current.loading).toBe(false));

        expect(result.current.error).toBeNull();
        expect(result.current.certificates.map((cert) => [cert.id, cert.type, cert.filename])).toEqual([
            ['new', 'backup', 'new.json'],
            ['old', 'wipe', 'old.json'],
        ]);
        expect(result.current.certificates[0].verifyUrl).toBe('http://localhost:8000/verify?cert_id=new');
    });

    it('skips files that are not certificates', async () => {
        vi.spyOn(console, 'warn').mockImplementation(() => {});
        fakeCertificates({
            [`${DIR}/good.json`]: { cert_id: 'good', cert_type: 'backup', created_at: '2026-01-01T00:00:00Z' },
            [`${DIR}/broken.json`]: '{ not json',
            [`${DIR}/other.json`]: { cert_id: 'x', cert_type: 'restore', created_at: '2026-01-01T00:00:00Z' },
        });
        const { result } = renderHook(() => useCertificates());

        await waitFor(() => expect(result.current.loading).toBe(false));

        expect(result.current.certificates.map((cert) => cert.id)).toEqual(['good']);
    });

    it('finds the PDF in the backups folder first, then next to the JSON', async () => {
        fakeCertificates(
            {
                [`${DIR}/a.json`]: { cert_id: 'a', cert_type: 'backup', created_at: '2026-01-02T00:00:00Z' },
                [`${DIR}/b.json`]: { cert_id: 'b', cert_type: 'backup', created_at: '2026-01-01T00:00:00Z' },
            },
            [`${HOME}/SecureWipe/backups/a.pdf`, `${DIR}/a.pdf`, `${DIR}/b.pdf`],
        );
        const { result } = renderHook(() => useCertificates());

        await waitFor(() => expect(result.current.loading).toBe(false));

        expect(result.current.certificates.map((cert) => cert.pdfPath)).toEqual([
            `${HOME}/SecureWipe/backups/a.pdf`,
            `${DIR}/b.pdf`,
        ]);
    });

    it('records a generated PDF on the right certificate', async () => {
        fakeCertificates({ [`${DIR}/a.json`]: { cert_id: 'a', cert_type: 'backup', created_at: '2026-01-01T00:00:00Z' } });
        const { result } = renderHook(() => useCertificates());
        await waitFor(() => expect(result.current.loading).toBe(false));

        act(() => result.current.setPdfPath('a', '/out/a.pdf'));

        expect(result.current.certificates[0].pdfPath).toBe('/out/a.pdf');
    });

    it('explains a folder it cannot read and lists nothing', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => HOME);
        tauri.on('list_cert_files', () => {
            throw 'Failed to read certificate directory: permission denied';
        });
        const { result } = renderHook(() => useCertificates());

        await waitFor(() => expect(result.current.loading).toBe(false));

        expect(result.current.certificates).toEqual([]);
        expect(result.current.error).toBe(
            'Could not read the certificates folder: Failed to read certificate directory: permission denied',
        );
    });
});
