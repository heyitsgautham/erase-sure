import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import Certificates from './Certificates';
import { installFakeTauri } from '../test/fakeTauri';
import { renderWithApp } from '../test/render';

describe('Certificates screen', () => {
    it('reports a certificate folder it cannot read instead of inventing a certificate', async () => {
        vi.spyOn(console, 'warn').mockImplementation(() => {});
        vi.spyOn(console, 'error').mockImplementation(() => {});
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => {
            throw 'Could not determine home directory';
        });

        renderWithApp(<Certificates />);

        expect(await screen.findByText('Error Loading Certificates')).toBeInTheDocument();
        expect(screen.getByText(/Could not determine home directory/)).toBeInTheDocument();
        expect(screen.queryByText(/397fad31/)).not.toBeInTheDocument();
    });
});
