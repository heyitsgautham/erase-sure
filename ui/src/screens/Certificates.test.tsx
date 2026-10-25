import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Certificates from './Certificates';
import Toast from '../components/Toast';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';

const HOME = '/home/me';
const DIR = `${HOME}/SecureWipe/certificates`;
const backupCert = { cert_id: 'b-1', cert_type: 'backup', created_at: '2026-02-01T10:00:00Z' };
const wipeCert = { cert_id: 'w-1', cert_type: 'wipe', created_at: '2026-01-01T10:00:00Z' };

function fakeDisk(files: Record<string, object>) {
    const tauri = installFakeTauri();
    tauri.on('get_home_dir', () => HOME);
    tauri.on('list_cert_files', () => Object.keys(files));
    tauri.on('read_file_content', (args) => JSON.stringify(files[String(args.filePath)]));
    tauri.on('file_exists', () => false);
    tauri.on('open_path', () => null);
    tauri.on('shell.open', () => null);
    return tauri;
}

function renderScreen() {
    return renderWithApp(
        <>
            <Certificates />
            <Toast />
        </>,
        { route: '/certificates' },
    );
}

describe('Certificates screen', () => {
    it('reports a certificate folder it cannot read instead of inventing a certificate', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => {
            throw 'Could not determine home directory';
        });

        renderScreen();

        expect(await screen.findByText('Error Loading Certificates')).toBeInTheDocument();
        expect(screen.getByText(/Could not determine home directory/)).toBeInTheDocument();
        expect(screen.queryByText(/397fad31/)).not.toBeInTheDocument();
    });

    it('retries after an error', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = fakeDisk({ [`${DIR}/b-1.json`]: backupCert });
        let fail = true;
        tauri.on('get_home_dir', () => {
            if (fail) throw 'busy';
            return HOME;
        });
        renderScreen();
        await screen.findByText('Error Loading Certificates');

        fail = false;
        await userEvent.click(screen.getByRole('button', { name: /Retry Loading/ }));

        expect(await screen.findByText('ID: b-1')).toBeInTheDocument();
    });

    it('invites a first backup when there are no certificates', async () => {
        fakeDisk({});
        renderScreen();

        expect(await screen.findByText('No Certificates Found')).toBeInTheDocument();
        await userEvent.click(screen.getByRole('button', { name: /Start Device Discovery/ }));
        expect(screen.getByTestId('location')).toHaveTextContent('/discover');
    });

    it('shows the newest certificate and switches to another', async () => {
        fakeDisk({ [`${DIR}/w-1.json`]: wipeCert, [`${DIR}/b-1.json`]: backupCert });
        renderScreen();

        expect(await screen.findByText('ID: b-1')).toBeInTheDocument();
        expect(screen.getByText('You have 2 certificates. Select one to view:')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /Backup Certificate #1/ })).toHaveClass('btn-primary');

        await userEvent.click(screen.getByRole('button', { name: /Wipe Certificate #2/ }));

        expect(screen.getByText('ID: w-1')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /Wipe Certificate #2/ })).toHaveClass('btn-primary');
    });

    it('opens the JSON file through the backend', async () => {
        const tauri = fakeDisk({ [`${DIR}/b-1.json`]: backupCert });
        renderScreen();
        await screen.findByText('ID: b-1');

        await userEvent.click(screen.getByRole('button', { name: /Open JSON Certificate/ }));

        await waitFor(() => expect(tauri.callsTo('open_path')[0]?.args).toEqual({ path: `${DIR}/b-1.json` }));
    });

    it('generates a PDF, then offers to open it', async () => {
        const tauri = fakeDisk({ [`${DIR}/b-1.json`]: backupCert });
        tauri.on('generate_pdf_for_cert', () => `${HOME}/SecureWipe/backups/b-1.pdf`);
        renderScreen();
        await screen.findByText('ID: b-1');

        await userEvent.click(screen.getByRole('button', { name: /Generate PDF/ }));

        expect(await screen.findByRole('button', { name: /Open PDF Certificate/ })).toBeInTheDocument();
        expect(tauri.callsTo('generate_pdf_for_cert')[0].args).toMatchObject({ certJsonPath: `${DIR}/b-1.json` });
        expect(tauri.callsTo('open_path').slice(-1)[0]?.args).toEqual({ path: `${HOME}/SecureWipe/backups/b-1.pdf` });
    });

    it("shows the portal's verdict check by check", async () => {
        fakeDisk({ [`${DIR}/b-1.json`]: backupCert });
        vi.spyOn(globalThis, 'fetch').mockResolvedValue(
            new Response(
                JSON.stringify({
                    schema_valid: true,
                    signature_valid: null,
                    hash_valid: false,
                    chain_valid: null,
                    cert_summary: { cert_id: 'b-1', cert_type: 'backup', device_model: 'USB' },
                    computed: {},
                    errors: ['Hash mismatch'],
                }),
            ),
        );
        renderScreen();
        await screen.findByText('ID: b-1');

        await userEvent.click(screen.getByRole('button', { name: /Verify Online/ }));

        const modal = (await screen.findByText('Certificate Verification Result')).closest(
            '.modal-content',
        ) as HTMLElement;
        expect(within(modal).getByText('Certificate ID: b-1')).toBeInTheDocument();
        expect(within(modal).getByText('- N/A')).toBeInTheDocument();
        expect(within(modal).queryByText('Chain Valid:')).not.toBeInTheDocument();
        expect(within(modal).getByText(/Hash mismatch/)).toBeInTheDocument();
        expect(await screen.findByText('Certificate verification completed with issues')).toBeInTheDocument();

        await userEvent.click(within(modal).getByRole('button', { name: 'Close' }));
        expect(screen.queryByText('Certificate Verification Result')).not.toBeInTheDocument();
    });

    it('opens the backups folder', async () => {
        const tauri = fakeDisk({ [`${DIR}/b-1.json`]: backupCert });
        renderScreen();
        await screen.findByText('ID: b-1');

        await userEvent.click(screen.getByRole('button', { name: /Open Backups Folder/ }));

        await waitFor(() =>
            expect(tauri.callsTo('open_path')[0]?.args).toEqual({ path: `${HOME}/SecureWipe/backups` }),
        );
    });
});
