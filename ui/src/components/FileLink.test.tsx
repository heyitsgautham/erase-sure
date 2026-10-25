import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import FileLink from './FileLink';
import Toast from './Toast';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';

describe('FileLink', () => {
    it('shows the label with the path underneath', () => {
        renderWithApp(<FileLink path="/home/me/backup" label="Open Backup Folder" />);

        expect(screen.getByText('Open Backup Folder')).toBeInTheDocument();
        expect(screen.getByText('/home/me/backup')).toBeInTheDocument();
        expect(screen.getByText('📁')).toBeInTheDocument();
    });

    it('falls back to the path and picks an icon for the file type', () => {
        renderWithApp(<FileLink path="/home/me/cert.pdf" type="pdf" />);

        expect(screen.getByText('/home/me/cert.pdf')).toBeInTheDocument();
        expect(screen.getByText('📋')).toBeInTheDocument();
    });

    it('asks the desktop backend to open the path', async () => {
        const tauri = installFakeTauri();
        tauri.on('open_path', () => null);
        renderWithApp(
            <>
                <FileLink path="/home/me/cert.json" label="Certificate" type="json" />
                <Toast />
            </>,
        );

        await userEvent.click(screen.getByRole('button', { name: /Certificate/ }));

        await waitFor(() =>
            expect(tauri.callsTo('open_path')).toEqual([{ cmd: 'open_path', args: { path: '/home/me/cert.json' } }]),
        );
        expect(await screen.findByText('Opened cert.json')).toBeInTheDocument();
    });

    it("shows the backend's reason when the path cannot be opened", async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('open_path', () => {
            throw 'Invalid or non-existent path: /gone';
        });
        renderWithApp(
            <>
                <FileLink path="/gone" />
                <Toast />
            </>,
        );

        await userEvent.click(screen.getByRole('button', { name: /gone/ }));

        expect(await screen.findByText('Invalid or non-existent path: /gone')).toBeInTheDocument();
    });
});
