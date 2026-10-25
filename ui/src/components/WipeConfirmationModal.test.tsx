import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import WipeConfirmationModal from './WipeConfirmationModal';
import { safeUsb } from '../test/devices';

function renderModal(overrides: Partial<Parameters<typeof WipeConfirmationModal>[0]> = {}) {
    const props = {
        device: safeUsb,
        policy: 'PURGE',
        onConfirm: vi.fn(),
        onCancel: vi.fn(),
        isOpen: true,
        ...overrides,
    };
    render(<WipeConfirmationModal {...props} />);
    return props;
}

describe('WipeConfirmationModal', () => {
    it('renders nothing while closed', () => {
        renderModal({ isOpen: false });
        expect(screen.queryByText('DESTRUCTIVE WIPE CONFIRMATION')).not.toBeInTheDocument();
    });

    it('keeps the wipe button disabled until the exact phrase with the serial is typed', async () => {
        const user = userEvent.setup();
        const { onConfirm } = renderModal();
        const input = screen.getByPlaceholderText('Type the confirmation text above...');
        const execute = screen.getByRole('button', { name: /EXECUTE WIPE/ });

        expect(screen.getByText(`WIPE ${safeUsb.serial}`)).toBeInTheDocument();
        expect(execute).toBeDisabled();

        await user.type(input, 'wipe ' + safeUsb.serial);
        expect(execute).toBeDisabled();
        expect(
            screen.getByText('Input does not match the required confirmation text shown above.'),
        ).toBeInTheDocument();

        await user.clear(input);
        await user.type(input, `WIPE ${safeUsb.serial}`);
        expect(execute).toBeEnabled();

        await user.click(execute);
        expect(onConfirm).toHaveBeenCalledWith(`WIPE ${safeUsb.serial}`);
        expect(input).toHaveValue('');
    });

    it('uses UNKNOWN when the device reports no serial', () => {
        renderModal({ device: { ...safeUsb, serial: '' } });
        expect(screen.getByText('WIPE UNKNOWN')).toBeInTheDocument();
    });

    it('cancels', async () => {
        const { onCancel, onConfirm } = renderModal();

        await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

        expect(onCancel).toHaveBeenCalledTimes(1);
        expect(onConfirm).not.toHaveBeenCalled();
    });
});
