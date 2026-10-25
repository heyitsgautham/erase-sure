import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { useApp } from '../contexts/AppContext';
import Backup from './Backup';
import Toast from '../components/Toast';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';
import { safeUsb, systemNvme } from '../test/devices';

const TS = '2026-01-01T00:00:00Z';

/** Prints the shared progress state so tests can read it even where the screen hides it. */
function ProgressProbe() {
    const { state } = useApp();
    return (
        <div data-testid="progress">
            {state.progress ? `${state.progress.title}|${state.progress.currentStep}` : 'none'}
        </div>
    );
}

const successfulBackup = ['Backup completed successfully!', 'Backup ID: 1234', 'Verification status: PASSED'];

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

    it('stops the fallback progress timer once the backup has finished', async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', (args) => {
            // Finish ten seconds in, after the fallback timer has ticked once.
            setTimeout(() => {
                for (const line of successfulBackup)
                    tauri.emit('securewipe://stdout', { line, ts: TS, stream: 'stdout' });
                tauri.emit('securewipe://exit', { code: 0, ts: TS });
            }, 10_000);
            return args && null;
        });
        renderWithApp(
            <>
                <Backup />
                <ProgressProbe />
            </>,
            { state: { selectedDevice: safeUsb } },
        );

        await act(async () => {
            screen.getByRole('button', { name: /Start Encrypted Backup/ }).click();
            await vi.advanceTimersByTimeAsync(10_500);
        });
        expect(screen.getByTestId('progress')).toHaveTextContent('Backup Complete!|5');

        // The screen clears the finished bar a few seconds later. The old timer kept firing every
        // 8 seconds and brought back an "in progress" bar at step 3, then 4, after the backup ended.
        await act(async () => {
            await vi.advanceTimersByTimeAsync(15_000);
        });
        expect(screen.getByTestId('progress')).toHaveTextContent('none');
        vi.useRealTimers();
    });

    it('keeps the failure on screen instead of resuming fake progress', async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', () => {
            setTimeout(() => {
                tauri.emit('securewipe://stderr', {
                    line: 'Permission denied (os error 13)',
                    ts: TS,
                    stream: 'stderr',
                });
                tauri.emit('securewipe://exit', { code: 1, ts: TS });
            }, 1_000);
            return null;
        });
        renderWithApp(
            <>
                <Backup />
                <ProgressProbe />
            </>,
            { state: { selectedDevice: safeUsb } },
        );

        await act(async () => {
            screen.getByRole('button', { name: /Start Encrypted Backup/ }).click();
            await vi.advanceTimersByTimeAsync(1_500);
        });
        expect(screen.getByTestId('progress')).toHaveTextContent('Backup Failed|0');

        await act(async () => {
            await vi.advanceTimersByTimeAsync(8_000);
        });
        expect(screen.getByTestId('progress')).toHaveTextContent('Backup Failed|0');
        vi.useRealTimers();
    });

    it('shows the progress bar while the backup runs', async () => {
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', () => {
            setTimeout(() => tauri.emit('securewipe://stdout', { line: 'Copying files', ts: TS, stream: 'stdout' }), 0);
            return null;
        });
        renderWithApp(<Backup />, { state: { selectedDevice: safeUsb } });

        screen.getByRole('button', { name: /Start Encrypted Backup/ }).click();

        expect(await screen.findByText('Encrypted Backup in Progress')).toBeInTheDocument();
        expect(await screen.findByText('Copying and encrypting files...')).toBeInTheDocument();
        expect(screen.getByText('Step 3 of 5')).toBeInTheDocument();
    });

    it('announces a finished backup once and then opens the certificates', async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: successfulBackup }));
        renderWithApp(
            <>
                <Backup />
                <Toast />
            </>,
            { state: { selectedDevice: safeUsb } },
        );

        await act(async () => {
            screen.getByRole('button', { name: /Start Encrypted Backup/ }).click();
            await vi.advanceTimersByTimeAsync(100);
        });

        const successToasts = [...document.querySelectorAll('.toast-item')].filter((toast) =>
            toast.textContent?.includes('Backup completed successfully'),
        );
        expect(successToasts).toHaveLength(1);
        expect(screen.getByText('Backup Complete!')).toBeInTheDocument();

        await act(async () => {
            await vi.advanceTimersByTimeAsync(4_000);
        });
        expect(screen.getByTestId('location')).toHaveTextContent('/certificates');
        vi.useRealTimers();
    });

    it('sends the user to discovery when no device is selected', async () => {
        renderWithApp(<Backup />);

        await userEvent.click(screen.getByRole('button', { name: /Go to Backup Device Discovery/ }));

        expect(screen.getByTestId('location')).toHaveTextContent('/backup-discover');
    });

    it('fills the destination from the folder dialog', async () => {
        const tauri = installFakeTauri();
        tauri.on('dialog.openDialog', () => '/mnt/usb/backups');
        renderWithApp(<Backup />, { state: { selectedDevice: safeUsb } });

        await userEvent.click(screen.getByRole('button', { name: /Browse$/ }));

        await waitFor(() =>
            expect(screen.getByPlaceholderText('~/SecureWipe/backups')).toHaveValue('/mnt/usb/backups'),
        );
        expect(tauri.callsTo('dialog.openDialog')[0].args).toMatchObject({
            options: { directory: true, multiple: false },
        });
    });

    it('refuses to start without a destination', async () => {
        const user = userEvent.setup();
        const tauri = installFakeTauri();
        renderWithApp(
            <>
                <Backup />
                <Toast />
            </>,
            { state: { selectedDevice: safeUsb } },
        );

        await user.clear(screen.getByPlaceholderText('~/SecureWipe/backups'));
        await user.click(screen.getByRole('button', { name: /Start Encrypted Backup/ }));

        expect(await screen.findByText('Please specify a backup destination')).toBeInTheDocument();
        expect(tauri.callsTo('run_securewipe')).toEqual([]);
    });

    it('backs up only the user folders from a system disk', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: successfulBackup }));
        renderWithApp(<Backup />, { state: { selectedDevice: systemNvme } });

        await userEvent.click(screen.getByRole('button', { name: /Start Encrypted Backup/ }));

        await waitFor(() => expect(tauri.callsTo('run_securewipe')).toHaveLength(1));
        const args = tauri.callsTo('run_securewipe')[0].args.args as string[];
        expect(args).toContain('--critical-ok');
        expect(args[args.indexOf('--paths') + 1]).toContain('$HOME/Documents');
        expect(args).toContain('--sign');
    });
});
