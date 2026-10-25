import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { AppProvider, useApp } from '../contexts/AppContext';
import { installFakeTauri } from '../test/fakeTauri';
import { useSecureWipe } from './useSecureWipe';

const wrapper = ({ children }: { children: ReactNode }) => <AppProvider>{children}</AppProvider>;

function useHookAndState() {
    return { wipe: useSecureWipe(), app: useApp() };
}

const usbJson = JSON.stringify([
    { name: '/dev/sdb', model: 'SanDisk Ultra USB', serial: 'X1', capacity_bytes: 1024, bus: 'usb', mountpoints: [], risk_level: 'SAFE' },
]);

describe('useSecureWipe.run', () => {
    it('collects output from a CLI that finishes before run_securewipe returns', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: ['hello'], stderr: ['warn'], code: 3 }));
        const { result } = renderHook(useHookAndState, { wrapper });

        let run: Awaited<ReturnType<typeof result.current.wipe.run>> | undefined;
        await act(async () => {
            run = await result.current.wipe.run(['discover', '--format', 'json']);
        });

        expect(run).toEqual({ exitCode: 3, stdout: ['hello'], stderr: ['warn'] });
        expect(result.current.wipe.running).toBe(false);
        expect(tauri.callsTo('run_securewipe')[0].args).toMatchObject({ args: ['discover', '--format', 'json'] });
    });

    it('reports a killed process as exit code -1', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ code: null }));
        const { result } = renderHook(useHookAndState, { wrapper });

        let exitCode: number | undefined;
        await act(async () => {
            exitCode = (await result.current.wipe.run(['discover'])).exitCode;
        });

        expect(exitCode).toBe(-1);
    });

    it('rejects and stops running when the backend refuses the arguments', async () => {
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', () => {
            throw "Subcommand 'rm' is not allowed";
        });
        const { result } = renderHook(useHookAndState, { wrapper });

        await act(async () => {
            await expect(result.current.wipe.run(['rm'])).rejects.toBe("Subcommand 'rm' is not allowed");
        });

        expect(result.current.wipe.running).toBe(false);
        expect(tauri.listenerCount('securewipe://exit')).toBe(0);
    });

    it('stops listening once the run is over', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: ['done'] }));
        const { result } = renderHook(useHookAndState, { wrapper });

        await act(async () => {
            await result.current.wipe.run(['discover']);
        });

        expect(tauri.listenerCount('securewipe://exit')).toBe(0);
        expect(tauri.listenerCount('securewipe://stdout')).toBe(0);
    });
});

describe('useSecureWipe.run timeout', () => {
    it('gives up after twenty minutes without an exit event', async () => {
        vi.useFakeTimers();
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', () => null);
        const { result } = renderHook(useHookAndState, { wrapper });

        let outcome: unknown;
        await act(async () => {
            const pending = result.current.wipe.run(['backup']).catch((error: Error) => error.message);
            await vi.advanceTimersByTimeAsync(20 * 60 * 1000);
            outcome = await pending;
        });
        vi.useRealTimers();

        expect(outcome).toBe('Process timeout');
        expect(result.current.wipe.running).toBe(false);
    });
});

describe('useSecureWipe.discover', () => {
    it('stores the discovered devices', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: [usbJson] }));
        const { result } = renderHook(useHookAndState, { wrapper });

        await act(async () => {
            await result.current.wipe.discover();
        });

        expect(result.current.app.state.devices).toMatchObject([{ path: '/dev/sdb', model: 'SanDisk Ultra USB' }]);
        expect(result.current.app.state.toasts).toMatchObject([{ message: 'Discovered 1 devices', type: 'success' }]);
    });
});
