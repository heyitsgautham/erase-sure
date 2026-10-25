import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import BackupDiscover from './BackupDiscover';
import DestructiveWipeDiscover from './DestructiveWipeDiscover';
import Discover from './Discover';
import WipePlanDiscover from './WipePlanDiscover';
import Toast from '../components/Toast';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';
import { safeUsb, systemNvme } from '../test/devices';

const cliDevices = JSON.stringify([
    { name: '/dev/nvme0n1', model: 'Samsung SSD 980', serial: 'S1', capacity_bytes: 500107862016, bus: 'nvme', mountpoints: ['/'], risk_level: 'CRITICAL' },
    { name: '/dev/sdb', model: 'SanDisk Ultra USB', serial: 'U1', capacity_bytes: 32010928128, bus: 'usb', mountpoints: [], risk_level: 'SAFE' },
]);

function renderScreen(screenElement: ReactElement, devices = [systemNvme, safeUsb]) {
    return renderWithApp(
        <>
            {screenElement}
            <Toast />
        </>,
        { state: { devices } },
    );
}

describe('device discovery', () => {
    it('scans through the CLI and lists what it found', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: [cliDevices] }));
        renderScreen(<BackupDiscover />, []);

        expect(screen.getByText('No Devices Found')).toBeInTheDocument();
        await userEvent.click(screen.getByRole('button', { name: /Scan Devices/ }));

        expect(await screen.findByText('Found 2 device(s): 1 safe, 1 critical')).toBeInTheDocument();
        expect(screen.getByText('Samsung SSD 980')).toBeInTheDocument();
        expect(screen.getByText('⚠️ Critical Devices Detected')).toBeInTheDocument();
        expect(tauri.callsTo('run_securewipe')[0].args.args).toEqual(['discover', '--format', 'json']);
    });

    it('scans from the empty state button too', async () => {
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stdout: [cliDevices] }));
        renderScreen(<Discover />, []);

        await userEvent.click(screen.getByRole('button', { name: /Start Device Scan/ }));

        await waitFor(() => expect(tauri.callsTo('run_securewipe')).toHaveLength(1));
    });

    it('selects a device and says what it may be used for', async () => {
        renderScreen(<DestructiveWipeDiscover />);

        await userEvent.click(screen.getByText('Samsung SSD 980'));

        expect(screen.getByText('Selected Device: Samsung SSD 980')).toBeInTheDocument();
        expect(screen.getByText('🚫 System Disk Selected')).toBeInTheDocument();
        expect(await screen.findByText('Selected Samsung SSD 980 (System disk - Wiping blocked for safety)')).toBeInTheDocument();

        await userEvent.click(screen.getByText('SanDisk Ultra USB'));
        expect(await screen.findByText('Selected SanDisk Ultra USB (Safe device selected)')).toBeInTheDocument();
        expect(screen.queryByText('🚫 System Disk Selected')).not.toBeInTheDocument();
    });

    it('goes back home and forgets the selection', async () => {
        renderWithApp(<BackupDiscover />, { state: { devices: [safeUsb], selectedDevice: safeUsb } });

        await userEvent.click(screen.getByRole('button', { name: /Back to Home/ }));

        expect(screen.getByTestId('location')).toHaveTextContent('/');
        expect(screen.queryByText('Selected Device: SanDisk Ultra USB')).not.toBeInTheDocument();
    });

    it('has no back button on the combined discovery screen', () => {
        renderScreen(<Discover />);
        expect(screen.queryByRole('button', { name: /Back to Home/ })).not.toBeInTheDocument();
    });
});

describe('next steps after discovery', () => {
    it('continues to the backup screen', async () => {
        renderScreen(<BackupDiscover />);
        await userEvent.click(screen.getByText('Samsung SSD 980'));

        await userEvent.click(screen.getByRole('button', { name: /Continue to Backup/ }));

        expect(screen.getByTestId('location')).toHaveTextContent('/backup');
    });

    it('opens the wipe plan for any device from the wipe plan screen', async () => {
        renderScreen(<WipePlanDiscover />);
        await userEvent.click(screen.getByText('Samsung SSD 980'));
        expect(screen.getByText(/Preview mode only/)).toBeInTheDocument();

        await userEvent.click(screen.getByRole('button', { name: /View Wipe Plan/ }));

        expect(screen.getByTestId('location')).toHaveTextContent('/wipe-plan');
    });

    it('blocks wipe planning for a system disk on the combined screen', async () => {
        renderScreen(<Discover />);
        await userEvent.click(screen.getByText('Samsung SSD 980'));

        await userEvent.click(screen.getByRole('button', { name: /View Wipe Plan/ }));

        expect(await screen.findByText('Wipe planning blocked for system disks. Use backup instead or boot from ISO.')).toBeInTheDocument();
        expect(screen.getByTestId('location')).toHaveTextContent(/^\/$/);
    });

    it('blocks a destructive wipe of a system disk', async () => {
        renderScreen(<DestructiveWipeDiscover />);
        await userEvent.click(screen.getByText('Samsung SSD 980'));

        await userEvent.click(screen.getByRole('button', { name: /Start Destructive Wipe/ }));

        expect(await screen.findByText('Cannot wipe CRITICAL devices unless running from ISO mode')).toBeInTheDocument();
        expect(screen.getByTestId('location')).toHaveTextContent(/^\/$/);
    });

    it('continues to the destructive wipe screen for a removable device', async () => {
        renderScreen(<DestructiveWipeDiscover />);
        await userEvent.click(screen.getByText('SanDisk Ultra USB'));

        await userEvent.click(screen.getByRole('button', { name: /Start Destructive Wipe/ }));

        expect(screen.getByTestId('location')).toHaveTextContent('/destructive-wipe');
    });
});
