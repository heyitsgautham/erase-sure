/**
 * Every test here runs against the fake Tauri backend. execute_destructive_wipe
 * is a canned handler, so nothing reaches the CLI or a block device.
 */
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import DestructiveWipe from './DestructiveWipe';
import Toast from '../components/Toast';
import { installFakeTauri } from '../test/fakeTauri';
import type { FakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';
import { safeUsb, systemNvme } from '../test/devices';

const TS = '2026-01-01T00:00:00Z';

function fakeWipeBackend(): FakeTauri {
    const tauri = installFakeTauri();
    tauri.on('validate_wipe_device', () => ({
        blockdevices: [{ name: 'sdb', serial: safeUsb.serial, is_critical: false }],
    }));
    tauri.on('execute_destructive_wipe', () => null);
    return tauri;
}

function renderScreen(device = safeUsb) {
    return renderWithApp(
        <>
            <DestructiveWipe />
            <Toast />
        </>,
        { state: { selectedDevice: device } },
    );
}

async function confirmWipe(user = userEvent.setup()) {
    await user.click(screen.getByRole('button', { name: /Start Destructive Wipe/ }));
    await user.type(screen.getByPlaceholderText('Type the confirmation text above...'), `WIPE ${safeUsb.serial}`);
    await user.click(screen.getByRole('button', { name: /EXECUTE WIPE/ }));
}

describe('DestructiveWipe screen', () => {
    it('asks for a device when none is selected', async () => {
        renderWithApp(<DestructiveWipe />);

        await userEvent.click(screen.getByRole('button', { name: /Go to Destructive Wipe Device Discovery/ }));

        expect(screen.getByTestId('location')).toHaveTextContent('/destructive-wipe-discover');
    });

    it('refuses to open the confirmation for a system disk', async () => {
        const tauri = fakeWipeBackend();
        renderScreen(systemNvme);

        await userEvent.click(screen.getByRole('button', { name: /Start Destructive Wipe/ }));

        expect(
            await screen.findByText('Cannot wipe CRITICAL devices unless running from ISO mode'),
        ).toBeInTheDocument();
        expect(screen.queryByText('DESTRUCTIVE WIPE CONFIRMATION')).not.toBeInTheDocument();
        expect(tauri.calls).toEqual([]);
    });

    it('sends the typed confirmation, policy and backup link to the backend', async () => {
        const user = userEvent.setup();
        const tauri = fakeWipeBackend();
        renderScreen();

        await user.click(screen.getByLabelText(/CLEAR/));
        await user.type(screen.getByPlaceholderText('Enter backup certificate ID to link this wipe'), 'b-1');
        await confirmWipe(user);

        await waitFor(() => expect(tauri.callsTo('execute_destructive_wipe')).toHaveLength(1));
        expect(tauri.callsTo('execute_destructive_wipe')[0].args).toEqual({
            confirmation: {
                device: '/dev/sdb',
                serial: safeUsb.serial,
                policy: 'clear',
                user_input: `WIPE ${safeUsb.serial}`,
            },
            backupCertId: 'b-1',
        });
        expect(screen.getByText(/Status: IN PROGRESS/)).toBeInTheDocument();
    });

    it('stops before wiping when the device check fails', async () => {
        const tauri = fakeWipeBackend();
        tauri.on('validate_wipe_device', () => {
            throw 'Device /dev/sdb not found or inaccessible';
        });
        renderScreen();

        await confirmWipe();

        expect(await screen.findByText(/Cannot access the selected device/)).toBeInTheDocument();
        expect(tauri.callsTo('execute_destructive_wipe')).toEqual([]);
    });

    it('reports success when the CLI exits with code 0', async () => {
        const tauri = fakeWipeBackend();
        renderScreen();
        await confirmWipe();
        await waitFor(() => expect(tauri.callsTo('execute_destructive_wipe')).toHaveLength(1));

        act(() => tauri.emit('securewipe://exit', { code: 0, ts: TS }));

        expect(await screen.findByText('Wipe operation completed successfully!')).toBeInTheDocument();
        expect(screen.getByText(/Status: COMPLETED/)).toBeInTheDocument();
    });

    it('explains a permission failure from the logs', async () => {
        const tauri = fakeWipeBackend();
        renderScreen();
        await confirmWipe();
        await waitFor(() => expect(tauri.callsTo('execute_destructive_wipe')).toHaveLength(1));

        act(() => {
            tauri.emit('securewipe://stderr', { line: 'Error: Permission denied (os error 13)', ts: TS });
        });
        act(() => tauri.emit('securewipe://exit', { code: 1, ts: TS }));

        expect(
            await screen.findByText(/Permission Error: SecureWipe needs administrator privileges/),
        ).toBeInTheDocument();
        expect(screen.getByText(/Status: FAILED/)).toBeInTheDocument();
    });

    it('treats a wipe killed by a signal as failed, not completed', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = fakeWipeBackend();
        renderScreen();
        await confirmWipe();
        await waitFor(() => expect(tauri.callsTo('execute_destructive_wipe')).toHaveLength(1));

        act(() => tauri.emit('securewipe://exit', { code: null, ts: TS }));

        expect(await screen.findByText(/Wipe operation was stopped before it finished/)).toBeInTheDocument();
        expect(screen.queryByText('Wipe operation completed successfully!')).not.toBeInTheDocument();
        expect(screen.getByText(/Status: FAILED/)).toBeInTheDocument();
    });
});
