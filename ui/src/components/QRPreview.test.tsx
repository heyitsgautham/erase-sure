import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import QRPreview from './QRPreview';

const toDataURL = vi.fn();
vi.mock('qrcode', () => ({
    default: { toDataURL: (...args: unknown[]) => toDataURL(...args) },
    toDataURL: (...args: unknown[]) => toDataURL(...args),
}));

describe('QRPreview', () => {
    beforeEach(() => {
        toDataURL.mockReset();
    });

    it('renders the QR code for the data at the requested size', async () => {
        toDataURL.mockResolvedValue('data:image/png;base64,QR');
        render(<QRPreview data="http://localhost:8000/verify?cert_id=a" title="Verification QR Code" size={120} />);

        const img = await screen.findByAltText('QR Code');
        expect(img).toHaveAttribute('src', 'data:image/png;base64,QR');
        expect(img).toHaveAttribute('width', '120');
        expect(screen.getByText('Verification QR Code')).toBeInTheDocument();
        expect(toDataURL).toHaveBeenCalledWith(
            'http://localhost:8000/verify?cert_id=a',
            expect.objectContaining({ width: 120, errorCorrectionLevel: 'M' }),
        );
    });

    it('shortens long data under the code', () => {
        toDataURL.mockReturnValue(new Promise(() => {}));
        const data = 'x'.repeat(60);
        render(<QRPreview data={data} />);

        expect(screen.getByText(`${'x'.repeat(50)}...`)).toBeInTheDocument();
        expect(screen.getByText('Generating QR...')).toBeInTheDocument();
    });

    it('says the code is unavailable instead of drawing a fake one', async () => {
        vi.spyOn(console, 'error').mockImplementation(() => {});
        toDataURL.mockRejectedValue(new Error('canvas unavailable'));
        render(<QRPreview data="http://localhost:8000/verify?cert_id=a" />);

        expect(await screen.findByText('QR code unavailable')).toBeInTheDocument();
        expect(screen.queryByAltText('QR Code')).not.toBeInTheDocument();
    });
});
