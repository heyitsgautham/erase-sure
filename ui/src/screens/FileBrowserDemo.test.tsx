import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import FileBrowserDemo from './FileBrowserDemo';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';

const listing = { entries: [{ name: 'a.txt', path: '/home/me/a.txt', is_dir: false, size: 1, modified: null }], total_size: 1, total_items: 1 };

describe('FileBrowserDemo screen', () => {
    it('shows the raw browse_folders answer and the browser', async () => {
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => '/home/me');
        tauri.on('browse_folders', () => listing);
        renderWithApp(<FileBrowserDemo />);

        expect(await screen.findByText('a.txt')).toBeInTheDocument();
        await userEvent.click(screen.getByRole('button', { name: 'Test browse_folders Command' }));

        expect(await screen.findByText(/"total_items": 1/)).toBeInTheDocument();
    });

    it('shows the error when the command fails', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => '/home/me');
        tauri.on('browse_folders', () => {
            throw 'Path does not exist: /home/me';
        });
        renderWithApp(<FileBrowserDemo />);

        await userEvent.click(screen.getByRole('button', { name: 'Test browse_folders Command' }));

        expect(await screen.findByText('Error: Path does not exist: /home/me')).toBeInTheDocument();
    });
});
