import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import FileBrowser from './FileBrowser';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';

const HOME = '/home/me';

function fakeDisk(sizes: Record<string, number> = {}) {
    const tauri = installFakeTauri();
    tauri.on('get_home_dir', () => HOME);
    tauri.on('browse_folders', (args) => {
        const dir = String(args.path);
        if (dir === `${HOME}/Documents`) {
            return { entries: [{ name: 'tax.pdf', path: `${dir}/tax.pdf`, is_dir: false, size: 1024, modified: null }], total_size: 1024, total_items: 1 };
        }
        return {
            entries: [
                { name: 'Documents', path: `${dir}/Documents`, is_dir: true, size: null, modified: '2026-01-01 09:00:00' },
                { name: 'notes.txt', path: `${dir}/notes.txt`, is_dir: false, size: 2048, modified: null },
            ],
            total_size: 2048,
            total_items: 2,
        };
    });
    tauri.on('calculate_selection_size', (args) => (args.paths as string[]).reduce((sum, path) => sum + (sizes[path] ?? 0), 0));
    return tauri;
}

describe('FileBrowser', () => {
    it("opens the user's home folder with breadcrumbs and totals", async () => {
        fakeDisk();
        renderWithApp(<FileBrowser onSelectionChange={() => {}} title="Pick files" />);

        expect(await screen.findByText('notes.txt')).toBeInTheDocument();
        expect(screen.getByText('Pick files')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'me' })).toBeInTheDocument();
        expect(screen.getByText('2 items • 2 KB')).toBeInTheDocument();
        expect(screen.getByText('Modified: 2026-01-01 09:00:00')).toBeInTheDocument();
    });

    it('opens a folder on click and comes back up', async () => {
        fakeDisk();
        renderWithApp(<FileBrowser onSelectionChange={() => {}} />);
        await screen.findByText('Documents');

        await userEvent.click(screen.getByText('Documents'));
        expect(await screen.findByText('tax.pdf')).toBeInTheDocument();

        await userEvent.click(screen.getByRole('button', { name: /Up/ }));
        expect(await screen.findByText('notes.txt')).toBeInTheDocument();
    });

    it('reports the selection with its size and warns past the limit', async () => {
        fakeDisk({ [`${HOME}/notes.txt`]: 2048, [`${HOME}/Documents`]: 5 * 1024 ** 3 });
        const onSelectionChange = vi.fn();
        renderWithApp(<FileBrowser onSelectionChange={onSelectionChange} maxSelectionSize={1024 ** 3} />);
        await screen.findByText('notes.txt');

        const [documentsBox, notesBox] = screen.getAllByRole('checkbox');
        await userEvent.click(notesBox);
        await waitFor(() => expect(onSelectionChange).toHaveBeenLastCalledWith([`${HOME}/notes.txt`]));
        expect(await screen.findByText('• 2 KB')).toBeInTheDocument();

        await userEvent.click(documentsBox);
        expect(await screen.findByText('⚠️ Selection exceeds 1 GB')).toBeInTheDocument();
        expect(screen.getByText('5 GB')).toBeInTheDocument();

        await userEvent.click(screen.getByRole('button', { name: 'Clear Selection' }));
        await waitFor(() => expect(onSelectionChange).toHaveBeenLastCalledWith([]));
    });

    it('selects everything in the folder', async () => {
        fakeDisk();
        const onSelectionChange = vi.fn();
        renderWithApp(<FileBrowser onSelectionChange={onSelectionChange} />);
        await screen.findByText('notes.txt');

        await userEvent.click(screen.getByRole('button', { name: 'Select All' }));

        await waitFor(() => expect(onSelectionChange).toHaveBeenLastCalledWith([`${HOME}/Documents`, `${HOME}/notes.txt`]));
        expect(screen.getByText('item(s) selected').parentElement).toHaveTextContent('2 item(s) selected');
    });

    it('hides checkboxes for kinds that may not be picked', async () => {
        fakeDisk();
        renderWithApp(<FileBrowser onSelectionChange={() => {}} allowFiles={false} />);
        const notes = await screen.findByText('notes.txt');

        const row = notes.closest('.flex.items-center.p-2') as HTMLElement;
        expect(within(row).queryByRole('checkbox')).not.toBeInTheDocument();
        expect(screen.getAllByRole('checkbox')).toHaveLength(1);
    });

    it('shows why the home folder could not be listed', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => {
            throw 'Could not determine home directory';
        });
        renderWithApp(<FileBrowser onSelectionChange={() => {}} />);

        expect(await screen.findByText('Error: Failed to load home directory: Could not determine home directory')).toBeInTheDocument();
        expect(screen.getByText('No items found')).toBeInTheDocument();
    });
});
