import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import DeviceCard from './DeviceCard';
import { safeUsb, systemNvme } from '../test/devices';

describe('DeviceCard', () => {
    it('shows model, capacity in GB, bus and path', () => {
        render(<DeviceCard device={safeUsb} />);

        expect(screen.getByText('SanDisk Ultra USB')).toBeInTheDocument();
        expect(screen.getByText('29.8 GB')).toBeInTheDocument();
        expect(screen.getByText('USB')).toBeInTheDocument();
        expect(screen.getByText('/dev/sdb')).toBeInTheDocument();
        expect(screen.getByText('SAFE')).toHaveClass('risk-badge', 'safe');
    });

    it('warns about a system disk and lists its mount points', () => {
        const { container } = render(<DeviceCard device={systemNvme} />);

        expect(screen.getByText('/, /boot')).toBeInTheDocument();
        expect(screen.getByText('System disk with active mount points')).toBeInTheDocument();
        expect(container.firstChild).toHaveClass('critical');
    });

    it('truncates long serials but keeps the full value in the tooltip', () => {
        const serial = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
        render(<DeviceCard device={{ ...safeUsb, serial }} />);

        const shown = screen.getByText('ABCDEFGHIJKLMNOPQRST...');
        expect(shown).toHaveAttribute('title', serial);
    });

    it('reports the device when clicked and marks the selected card', async () => {
        const onSelect = vi.fn();
        const { container } = render(<DeviceCard device={safeUsb} selected onSelect={onSelect} />);

        await userEvent.click(screen.getByText('SanDisk Ultra USB'));

        expect(onSelect).toHaveBeenCalledWith(safeUsb);
        expect(container.firstChild).toHaveClass('selected');
    });
});
