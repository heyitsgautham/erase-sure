import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import Navigation from './Navigation';
import { renderWithApp } from '../test/render';
import { breadcrumbsFor, ROUTE_TITLES } from '../lib/routes';

describe('breadcrumbsFor', () => {
    it('names every screen after Home', () => {
        for (const [path, title] of Object.entries(ROUTE_TITLES)) {
            if (path === '/') continue;
            expect(breadcrumbsFor(path)).toEqual([
                { label: 'Home', path: '/', current: false },
                { label: title, path, current: true },
            ]);
        }
    });

    it('shows Home alone on the home screen and on unknown paths', () => {
        expect(breadcrumbsFor('/')).toEqual([{ label: 'Home', path: '/', current: true }]);
        expect(breadcrumbsFor('/nowhere')).toEqual([{ label: 'Home', path: '/', current: false }]);
    });
});

describe('Navigation', () => {
    it('shows the current screen and goes home from the crumb or the title', async () => {
        renderWithApp(<Navigation />, { route: '/destructive-wipe' });

        expect(screen.getByText('Destructive Wipe')).toHaveClass('breadcrumb-current');
        await userEvent.click(screen.getByText('Home'));
        expect(screen.getByTestId('location')).toHaveTextContent(/^\/$/);
    });

    it('goes home from the app title', async () => {
        renderWithApp(<Navigation />, { route: '/backup' });

        await userEvent.click(screen.getByTitle('Go to Home'));

        expect(screen.getByTestId('location')).toHaveTextContent(/^\/$/);
    });
});
