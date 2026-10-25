import { act, renderHook, waitFor } from '@testing-library/react';
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
    {
        name: '/dev/sdb',
        model: 'SanDisk Ultra USB',
        serial: 'X1',
        capacity_bytes: 1024,
        bus: 'usb',
        mountpoints: [],
        risk_level: 'SAFE',
    },
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

    it('reports malformed discover output instead of showing broken devices', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: [JSON.stringify([{ model: 'No path' }])] }));
        const { result } = renderHook(useHookAndState, { wrapper });

        await act(async () => {
            await expect(result.current.wipe.discover()).rejects.toThrow('Unexpected device list from securewipe');
        });

        expect(result.current.app.state.devices).toEqual([]);
        expect(result.current.app.state.toasts[0]).toMatchObject({ type: 'error' });
    });
});

describe('useSecureWipe.verifyOnline', () => {
    const verified = {
        schema_valid: true,
        signature_valid: true,
        hash_valid: true,
        chain_valid: null,
        cert_summary: { cert_id: 'abc', cert_type: 'backup', device_model: null },
        computed: {},
        errors: [],
    };

    it("returns the portal's validated answer", async () => {
        const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify(verified)));
        const { result } = renderHook(useHookAndState, { wrapper });

        const outcome = await result.current.wipe.verifyOnline({ cert_id: 'abc' });

        expect(outcome).toEqual({ kind: 'verified', result: verified });
        expect(fetchMock).toHaveBeenCalledWith(
            'http://localhost:8000/verify',
            expect.objectContaining({ method: 'POST', body: '{"cert_id":"abc"}' }),
        );
    });

    it.each([
        ['an HTTP error', () => new Response('nope', { status: 500, statusText: 'Server Error' })],
        ['a response of the wrong shape', () => new Response(JSON.stringify({ ok: true }))],
    ])('opens the portal page for the certificate after %s', async (_, response) => {
        vi.spyOn(console, 'warn').mockImplementation(() => {});
        vi.spyOn(globalThis, 'fetch').mockResolvedValue(response());
        const tauri = installFakeTauri();
        tauri.on('shell.open', () => null);
        const { result } = renderHook(useHookAndState, { wrapper });

        const outcome = await result.current.wipe.verifyOnline({ cert_id: 'a b' });

        expect(outcome).toEqual({ kind: 'opened-portal' });
        expect(tauri.callsTo('shell.open')[0].args).toMatchObject({
            path: 'http://localhost:8000/verify?cert_id=a%20b',
        });
    });
});

describe('useSecureWipe.cancel', () => {
    it('cancels the running session, and says so when the backend cannot', async () => {
        vi.spyOn(console, 'warn').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', () => null);
        tauri.on('cancel_securewipe', () => null);
        const { result } = renderHook(useHookAndState, { wrapper });

        act(() => {
            void result.current.wipe.run(['backup'], 'session-1').catch(() => {});
        });
        await waitFor(() => expect(result.current.wipe.running).toBe(true));

        await act(() => result.current.wipe.cancel());
        expect(tauri.callsTo('cancel_securewipe')[0].args).toEqual({ sessionId: 'session-1' });
        expect(result.current.app.state.toasts.slice(-1)[0]).toMatchObject({ message: 'Process cancelled' });

        tauri.on('cancel_securewipe', () => {
            throw 'Session not found';
        });
        await act(() => result.current.wipe.cancel());
        expect(result.current.app.state.toasts.slice(-1)[0]).toMatchObject({
            message: 'Failed to cancel process',
            type: 'error',
        });
    });
});
