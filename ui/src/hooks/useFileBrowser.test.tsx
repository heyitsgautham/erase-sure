import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useFileBrowser } from './useFileBrowser';
import { installFakeTauri } from '../test/fakeTauri';
import type { FakeTauri } from '../test/fakeTauri';

const listing = (dir: string) => ({
    entries: [
        { name: 'Documents', path: `${dir}/Documents`, is_dir: true, size: null, modified: null },
        { name: 'notes.txt', path: `${dir}/notes.txt`, is_dir: false, size: 2048, modified: '2026-01-01 00:00:00' },
    ],
    total_size: 2048,
    total_items: 2,
});

function fakeDisk(): FakeTauri {
    const tauri = installFakeTauri();
    tauri.on('get_home_dir', () => '/Users/gk');
    tauri.on('browse_folders', (args) => listing(String(args.path)));
    return tauri;
}

describe('useFileBrowser', () => {
    it("opens the user's real home folder", async () => {
        const tauri = fakeDisk();
        const { result } = renderHook(() => useFileBrowser());

        await act(() => result.current.goHome());

        expect(result.current.currentPath).toBe('/Users/gk');
        expect(tauri.callsTo('browse_folders')[0].args).toEqual({ path: '/Users/gk' });
        expect(result.current.listing?.entries.map((entry) => entry.name)).toEqual(['Documents', 'notes.txt']);
    });

    it('goes up one folder and stops at the root', async () => {
        fakeDisk();
        const { result } = renderHook(() => useFileBrowser());

        await act(() => result.current.navigateTo('/Users/gk/Documents'));
        await act(() => result.current.navigateUp());
        expect(result.current.currentPath).toBe('/Users/gk');

        await act(() => result.current.navigateTo('/Users'));
        await act(() => result.current.navigateUp());
        expect(result.current.currentPath).toBe('/');
    });

    it('selects, deselects, selects all and clears', async () => {
        fakeDisk();
        const { result } = renderHook(() => useFileBrowser());
        await act(() => result.current.navigateTo('/data'));

        act(() => result.current.toggleSelection('/data/notes.txt'));
        expect(result.current.isSelected('/data/notes.txt')).toBe(true);

        act(() => result.current.toggleSelection('/data/notes.txt'));
        expect(result.current.selectedPaths).toEqual([]);

        act(() => result.current.selectAll());
        expect(result.current.selectedPaths).toEqual(['/data/Documents', '/data/notes.txt']);

        act(() => result.current.clearSelection());
        expect(result.current.selectedPaths).toEqual([]);
    });

    it('clears the selection when the folder changes', async () => {
        fakeDisk();
        const { result } = renderHook(() => useFileBrowser());
        await act(() => result.current.navigateTo('/data'));
        act(() => result.current.toggleSelection('/data/notes.txt'));

        await act(() => result.current.navigateTo('/data/Documents'));

        expect(result.current.selectedPaths).toEqual([]);
    });

    it('asks the backend for the size of the selection', async () => {
        const tauri = fakeDisk();
        tauri.on('calculate_selection_size', () => 4096);
        const { result } = renderHook(() => useFileBrowser());

        expect(await result.current.calculateSelectionSize()).toBe(0);

        act(() => result.current.toggleSelection('/data/notes.txt'));
        let size = 0;
        await act(async () => {
            size = await result.current.calculateSelectionSize();
        });

        expect(size).toBe(4096);
        expect(tauri.callsTo('calculate_selection_size')[0].args).toEqual({ paths: ['/data/notes.txt'] });
    });

    it('shows a folder that cannot be read as an error and keeps the old listing', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = fakeDisk();
        const { result } = renderHook(() => useFileBrowser());
        await act(() => result.current.navigateTo('/data'));

        tauri.on('browse_folders', () => {
            throw 'Failed to read directory: permission denied';
        });
        await act(() => result.current.navigateTo('/root'));

        expect(result.current.error).toBe('Failed to read directory: permission denied');
        expect(result.current.currentPath).toBe('/data');
        expect(result.current.isLoading).toBe(false);
    });

    it('formats sizes in binary units', () => {
        const { result } = renderHook(() => useFileBrowser());

        expect(result.current.formatFileSize(0)).toBe('0 B');
        expect(result.current.formatFileSize(1536)).toBe('1.5 KB');
        expect(result.current.formatFileSize(5 * 1024 ** 3)).toBe('5 GB');
    });
});
