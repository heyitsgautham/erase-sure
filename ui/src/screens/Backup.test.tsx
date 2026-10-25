import { act, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import Backup from './Backup';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';
import { safeUsb } from '../test/devices';

const TS = '2026-01-01T00:00:00Z';

describe('Backup screen', () => {
    it('shows each CLI output line once in the log viewer', async () => {
        const tauri = installFakeTauri();
        renderWithApp(<Backup />, { state: { selectedDevice: safeUsb } });
        await waitFor(() => expect(tauri.listenerCount('securewipe://stdout')).toBeGreaterThan(0));

        act(() => tauri.emit('securewipe://stdout', { line: 'Copying files', ts: TS, stream: 'stdout' }));
        await screen.findByText(/Copying files/);
        await act(async () => {});
        act(() => tauri.emit('securewipe://stdout', { line: 'Verifying integrity', ts: TS, stream: 'stdout' }));
        await screen.findByText(/Verifying integrity/);

        expect(screen.getAllByText(/Copying files/)).toHaveLength(1);
        expect(screen.getAllByText(/Verifying integrity/)).toHaveLength(1);
    });

    it('does not show output left over from device discovery', () => {
        installFakeTauri();
        renderWithApp(<Backup />, {
            state: { selectedDevice: safeUsb, logs: [{ line: '[{"name":"/dev/sdb"}]', ts: TS, stream: 'stdout' }] },
        });

        expect(screen.queryByText('Backup Progress')).not.toBeInTheDocument();
    });
});
