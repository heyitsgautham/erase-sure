import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import ErrorBoundary from './ErrorBoundary';

let shouldThrow = true;

function Flaky() {
    if (shouldThrow) throw new Error('certificate file vanished');
    return <p>Recovered</p>;
}

describe('ErrorBoundary', () => {
    it('shows the error with a way to try again, and logs it', async () => {
        const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
        shouldThrow = true;
        render(
            <ErrorBoundary what="The certificate list">
                <Flaky />
            </ErrorBoundary>,
        );

        expect(screen.getByText('Something went wrong')).toBeInTheDocument();
        expect(screen.getByText('The certificate list encountered an error:')).toBeInTheDocument();
        expect(screen.getByText('Error: certificate file vanished')).toBeInTheDocument();
        const logged = consoleError.mock.calls
            .map((call) => String(call[0]))
            .find((line) => line.includes('render failed'));
        expect(JSON.parse(logged ?? '{}')).toMatchObject({
            module: 'error-boundary',
            data: { what: 'The certificate list' },
        });

        shouldThrow = false;
        await userEvent.click(screen.getByRole('button', { name: 'Try again' }));

        expect(screen.getByText('Recovered')).toBeInTheDocument();
    });

    it('renders its children when nothing fails', () => {
        shouldThrow = false;
        render(
            <ErrorBoundary>
                <Flaky />
            </ErrorBoundary>,
        );

        expect(screen.getByText('Recovered')).toBeInTheDocument();
    });
});
