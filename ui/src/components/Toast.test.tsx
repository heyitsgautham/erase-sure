import { act, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent } from '@testing-library/react';
import Toast from './Toast';
import { renderWithApp } from '../test/render';

describe('Toast', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('renders nothing when there are no toasts', () => {
        renderWithApp(<Toast />);
        expect(screen.queryByRole('button', { name: '×' })).not.toBeInTheDocument();
    });

    it('shows each toast with its type and removes it when closed', () => {
        renderWithApp(<Toast />, {
            state: { toasts: [{ id: 'a', type: 'error', message: 'Backup failed' }] },
        });

        const toast = screen.getByText('Backup failed').parentElement;
        expect(toast).toHaveClass('alert-error');

        fireEvent.click(screen.getByRole('button', { name: '×' }));
        expect(toast).toHaveClass('toast-fade-out');

        act(() => {
            vi.advanceTimersByTime(300);
        });
        expect(screen.queryByText('Backup failed')).not.toBeInTheDocument();
    });

    it('dismisses a toast on its own after four seconds', () => {
        renderWithApp(<Toast />, {
            state: { toasts: [{ id: 'a', type: 'info', message: 'Scanning' }] },
        });

        act(() => {
            vi.advanceTimersByTime(3999);
        });
        expect(screen.getByText('Scanning')).toBeInTheDocument();

        act(() => {
            vi.advanceTimersByTime(301);
        });
        expect(screen.queryByText('Scanning')).not.toBeInTheDocument();
    });
});
