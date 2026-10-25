import { act, render, renderHook, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { AppProvider, MAX_LOG_LINES, useApp } from './AppContext';
import type { LogEvent } from '../types/securewipe';
import Home from '../screens/Home';
import { safeUsb } from '../test/devices';

const line = (text: string): LogEvent => ({ line: text, ts: '2026-01-01T00:00:00Z', stream: 'stdout' });

const wrapper = ({ children }: { children: ReactNode }) => <AppProvider>{children}</AppProvider>;

describe('AppContext', () => {
    it('throws when used outside the provider', () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        expect(() => renderHook(() => useApp())).toThrow('useApp must be used within an AppProvider');
    });

    it('selects and clears a device', () => {
        const { result } = renderHook(() => useApp(), { wrapper });

        act(() => result.current.dispatch({ type: 'SELECT_DEVICE', payload: safeUsb }));
        expect(result.current.state.selectedDevice).toEqual(safeUsb);

        act(() => result.current.clearDeviceSelection());
        expect(result.current.state.selectedDevice).toBeNull();
    });

    it('appends and clears log lines', () => {
        const { result } = renderHook(() => useApp(), { wrapper });

        act(() => result.current.addLog(line('first')));
        act(() => result.current.addLog(line('second')));
        expect(result.current.state.logs.map((log) => log.line)).toEqual(['first', 'second']);

        act(() => result.current.clearLogs());
        expect(result.current.state.logs).toEqual([]);
    });

    it('keeps only the newest log lines', () => {
        const { result } = renderHook(() => useApp(), { wrapper });

        act(() => {
            for (let i = 0; i <= MAX_LOG_LINES; i++) result.current.addLog(line(`line ${i}`));
        });

        expect(result.current.state.logs).toHaveLength(MAX_LOG_LINES);
        expect(result.current.state.logs[0].line).toBe('line 1');
    });

    it('removes a toast after its duration', () => {
        vi.useFakeTimers();
        const { result } = renderHook(() => useApp(), { wrapper });

        act(() => result.current.addToast('Saved', 'success', 1000));
        expect(result.current.state.toasts).toMatchObject([{ message: 'Saved', type: 'success' }]);

        act(() => {
            vi.advanceTimersByTime(1000);
        });
        expect(result.current.state.toasts).toEqual([]);
        vi.useRealTimers();
    });

    it('keeps a toast with no duration until it is removed', () => {
        const { result } = renderHook(() => useApp(), { wrapper });

        act(() => result.current.addToast('Pinned', 'warning', 0));
        const [toast] = result.current.state.toasts;
        act(() => result.current.dispatch({ type: 'REMOVE_TOAST', payload: toast.id }));

        expect(result.current.state.toasts).toEqual([]);
    });

    it('keeps its helper functions stable across state changes', () => {
        const { result } = renderHook(() => useApp(), { wrapper });
        const first = result.current;

        act(() => result.current.addLog(line('line')));

        expect(result.current.addLog).toBe(first.addLog);
        expect(result.current.addToast).toBe(first.addToast);
        expect(result.current.clearLogs).toBe(first.clearLogs);
        expect(result.current.clearDeviceSelection).toBe(first.clearDeviceSelection);
    });

    it('lets the home screen clear the selection without a render loop', () => {
        const errors = vi.spyOn(console, 'error').mockImplementation(() => {});

        render(
            <MemoryRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
                <AppProvider initial={{ selectedDevice: safeUsb }}>
                    <Home />
                </AppProvider>
            </MemoryRouter>,
        );

        expect(screen.getByText('SecureWipe')).toBeInTheDocument();
        const loopWarnings = errors.mock.calls.filter((call) => String(call[0]).includes('Maximum update depth'));
        expect(loopWarnings).toEqual([]);
    });
});
