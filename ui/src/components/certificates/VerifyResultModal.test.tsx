import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import VerifyResultModal from './VerifyResultModal';
import type { VerifyResult } from '../../types/securewipe';

const base: VerifyResult = {
    schema_valid: true,
    signature_valid: true,
    hash_valid: true,
    chain_valid: null,
    cert_summary: { cert_id: 'abc', cert_type: 'backup', device_model: 'USB' },
    computed: {},
    errors: [],
};

const valueOf = (label: string) => screen.getByText(`${label}:`).nextElementSibling as HTMLElement;

describe('VerifyResultModal', () => {
    it('marks passing checks green and hides the chain check when there is none', () => {
        render(<VerifyResultModal result={base} onClose={() => {}} />);

        expect(valueOf('Schema Valid')).toHaveTextContent('✓ Yes');
        expect(valueOf('Schema Valid')).toHaveClass('text-green-600');
        expect(screen.queryByText('Chain Valid:')).not.toBeInTheDocument();
        expect(screen.queryByText('Issues Found:')).not.toBeInTheDocument();
    });

    it('marks failures red, unknown signature and hash checks grey, and lists the issues', () => {
        render(
            <VerifyResultModal
                result={{ ...base, schema_valid: null, signature_valid: null, hash_valid: false, chain_valid: false, errors: ['Bad hash'] }}
                onClose={() => {}}
            />,
        );

        expect(valueOf('Schema Valid')).toHaveTextContent('✗ No');
        expect(valueOf('Signature Valid')).toHaveTextContent('- N/A');
        expect(valueOf('Signature Valid')).toHaveClass('text-gray-500');
        expect(valueOf('Hash Valid')).toHaveClass('text-red-600');
        expect(valueOf('Chain Valid')).toHaveTextContent('✗ No');
        expect(screen.getByText('• Bad hash')).toHaveClass('text-red-600');
    });

    it('closes', async () => {
        const onClose = vi.fn();
        render(<VerifyResultModal result={base} onClose={onClose} />);

        await userEvent.click(screen.getByRole('button', { name: 'Close' }));

        expect(onClose).toHaveBeenCalledTimes(1);
    });
});
