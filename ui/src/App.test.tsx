import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';
import App from './App';

function renderAt(path: string) {
    return render(
        <MemoryRouter initialEntries={[path]}>
            <App />
        </MemoryRouter>,
    );
}

describe('App', () => {
    it.each([
        ['/', 'Mission Statement'],
        [
            '/discover',
            'Scan and analyze storage devices for secure wiping operations. Select a device to continue with backup or wipe planning.',
        ],
        ['/backup', 'No Device Selected'],
        ['/wipe-plan', 'No Device Selected'],
        ['/destructive-wipe', 'No Device Selected'],
    ])('routes %s to its screen', (path, text) => {
        renderAt(path);
        expect(screen.getByText(text)).toBeInTheDocument();
    });

    it('moves between screens from the home page', async () => {
        renderAt('/');

        await userEvent.click(screen.getByRole('button', { name: /Backup & Wipe/ }));

        expect(screen.getByText('📦 Backup Device Discovery')).toBeInTheDocument();
        expect(screen.getByText('Backup Device Discovery', { selector: '.breadcrumb-current' })).toBeInTheDocument();
    });
});
