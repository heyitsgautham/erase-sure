import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import Home from './Home';
import { renderWithApp } from '../test/render';

describe('Home screen', () => {
    it.each([
        [/Backup & Wipe/, '/backup-discover'],
        [/Wipe Plan Only/, '/wipe-plan-discover'],
        [/Destructive Wipe/, '/destructive-wipe-discover'],
        [/File Browser/, '/file-browser-demo'],
    ])('opens the right flow from %s', async (name, path) => {
        renderWithApp(<Home />);

        await userEvent.click(screen.getByRole('button', { name }));

        expect(screen.getByTestId('location')).toHaveTextContent(path);
    });
});
