import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import WipePlan from './WipePlan';
import Toast from '../components/Toast';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';
import { safeUsb } from '../test/devices';
import type { WipePlan as Plan } from '../contexts/AppContext';

const planFor = (device: string, method: string) =>
    JSON.stringify({
        device,
        policy: 'PURGE',
        main_method: method,
        hpa_dco_clear: true,
        verification: { samples: 128 },
        blocked: false,
        reason: null,
    });

function fakePlanner(method = 'nvme_sanitize_crypto_erase') {
    const tauri = installFakeTauri();
    tauri.cli((args) => ({ stdout: [planFor(args[2], method)] }));
    return tauri;
}

describe('WipePlan screen', () => {
    it('plans the selected device as soon as it opens, in planning mode only', async () => {
        const tauri = fakePlanner();
        renderWithApp(
            <>
                <WipePlan />
                <Toast />
            </>,
            { state: { selectedDevice: safeUsb } },
        );

        expect(await screen.findByText('nvme_sanitize_crypto_erase')).toBeInTheDocument();
        expect(screen.getByText('✓ Enabled')).toHaveClass('text-green-600');
        expect(screen.getByText('128 random locations')).toBeInTheDocument();
        const args = tauri.callsTo('run_securewipe')[0].args.args as string[];
        expect(args).toEqual(['wipe', '--device', '/dev/sdb', '--format', 'json', '--samples', '128']);
        expect(args).not.toContain('--danger-allow-wipe');
    });

    it('replans when the stored plan belongs to another device', async () => {
        const tauri = fakePlanner('overwrite_random');
        const stale: Plan = {
            device_path: '/dev/nvme0n1',
            policy: 'PURGE',
            main_method: 'nvme_sanitize_crypto_erase',
            hpa_dco_clear: false,
            verification: { samples: 128 },
            blocked: false,
        };
        renderWithApp(<WipePlan />, { state: { selectedDevice: safeUsb, wipePlan: stale } });

        expect(await screen.findByText('overwrite_random')).toBeInTheDocument();
        expect(screen.queryByText('nvme_sanitize_crypto_erase')).not.toBeInTheDocument();
        expect(tauri.callsTo('run_securewipe')).toHaveLength(1);
    });

    it('switches between the readable and JSON views', async () => {
        fakePlanner();
        renderWithApp(<WipePlan />, { state: { selectedDevice: safeUsb } });
        await screen.findByText('nvme_sanitize_crypto_erase');

        await userEvent.click(screen.getByRole('button', { name: 'JSON View' }));

        expect(screen.getByText(/"device_path": "\/dev\/sdb"/)).toBeInTheDocument();
    });

    it('says so when planning fails', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.cli(() => ({ stderr: ['Error: device not found'], code: 1 }));
        renderWithApp(
            <>
                <WipePlan />
                <Toast />
            </>,
            { state: { selectedDevice: safeUsb } },
        );

        expect(
            await screen.findByText('Failed to create wipe plan. Please check device connection.'),
        ).toBeInTheDocument();
    });

    it('asks for a device when none is selected', async () => {
        renderWithApp(<WipePlan />);

        await userEvent.click(screen.getByRole('button', { name: /Device Discovery/ }));

        await waitFor(() => expect(screen.getByTestId('location')).toHaveTextContent('/wipe-plan-discover'));
    });
});
