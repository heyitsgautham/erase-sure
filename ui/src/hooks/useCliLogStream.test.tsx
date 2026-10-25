import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { describe, expect, it, vi } from 'vitest';
import { AppProvider, useApp } from '../contexts/AppContext';
import { installFakeTauri } from '../test/fakeTauri';
import { isHiddenLine, useCliLogStream } from './useCliLogStream';

const TS = '2026-01-01T00:00:00Z';
const wrapper = ({ children }: { children: ReactNode }) => <AppProvider>{children}</AppProvider>;

function useStreamAndLogs() {
    useCliLogStream();
    return useApp().state.logs;
}

describe('useCliLogStream', () => {
    it('copies stdout and stderr lines into the app log with their stream', async () => {
        const tauri = installFakeTauri();
        const { result } = renderHook(useStreamAndLogs, { wrapper });
        await waitFor(() => expect(tauri.listenerCount('securewipe://stderr')).toBe(1));

        act(() => {
            tauri.emit('securewipe://stdout', { line: 'scanning', ts: TS });
            tauri.emit('securewipe://stderr', { line: 'lsblk failed', ts: TS });
        });

        expect(result.current).toEqual([
            { line: 'scanning', ts: TS, stream: 'stdout' },
            { line: 'lsblk failed', ts: TS, stream: 'stderr' },
        ]);
    });

    it('leaves out the missing signing key warning', async () => {
        const tauri = installFakeTauri();
        const { result } = renderHook(useStreamAndLogs, { wrapper });
        await waitFor(() => expect(tauri.listenerCount('securewipe://stderr')).toBe(1));

        act(() => tauri.emit('securewipe://stderr', { line: 'Failed to load signing key: not found', ts: TS }));

        expect(result.current).toEqual([]);
    });

    it('unsubscribes on unmount', async () => {
        const tauri = installFakeTauri();
        const { unmount } = renderHook(useStreamAndLogs, { wrapper });
        await waitFor(() => expect(tauri.listenerCount('securewipe://stdout')).toBe(1));

        unmount();

        await waitFor(() => expect(tauri.listenerCount('securewipe://stdout')).toBe(0));
        expect(tauri.listenerCount('securewipe://stderr')).toBe(0);
    });

    it('drops a subscription that finishes after the component unmounted', async () => {
        const tauri = installFakeTauri();
        const { unmount } = renderHook(useStreamAndLogs, { wrapper });

        unmount();

        await new Promise((resolve) => setTimeout(resolve, 0));
        expect(tauri.listenerCount('securewipe://stdout')).toBe(0);
        expect(tauri.listenerCount('securewipe://stderr')).toBe(0);
    });
});

describe('useCliLogStream without a backend', () => {
    it('logs a warning instead of rejecting', async () => {
        mockIPC(() => {
            throw new Error('no backend');
        });
        const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});

        renderHook(useStreamAndLogs, { wrapper });

        await waitFor(() => expect(warn).toHaveBeenCalledTimes(2));
        expect(JSON.parse(warn.mock.calls[0][0] as string)).toMatchObject({
            module: 'cli-log-stream',
            msg: 'CLI log stream unavailable',
            error: { message: 'no backend' },
        });
    });
});

describe('isHiddenLine', () => {
    it('hides signing key noise on stderr only', () => {
        expect(isHiddenLine({ line: 'SECUREWIPE_SIGN_KEY_PATH not set', ts: TS, stream: 'stderr' })).toBe(true);
        expect(isHiddenLine({ line: 'SECUREWIPE_SIGN_KEY_PATH not set', ts: TS, stream: 'stdout' })).toBe(false);
        expect(isHiddenLine({ line: 'Permission denied', ts: TS, stream: 'stderr' })).toBe(false);
    });
});
