import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import Progress from './Progress';

describe('Progress', () => {
    it('shows the given percentage and step', () => {
        render(<Progress title="Backup" currentStep={2} totalSteps={5} currentStepName="Encrypting" percentage={40} />);

        expect(screen.getByText('Backup')).toBeInTheDocument();
        expect(screen.getByText('40%')).toBeInTheDocument();
        expect(screen.getByText('Step 2 of 5')).toBeInTheDocument();
        expect(screen.getByText('Encrypting')).toBeInTheDocument();
    });

    it('derives the percentage from the step when none is given', () => {
        const { container } = render(<Progress title="Wipe" currentStep={1} totalSteps={3} currentStepName="Pass 1" />);

        expect(screen.getByText('33%')).toBeInTheDocument();
        expect(container.querySelector('.progress-bar')).toHaveStyle({ width: '33%' });
    });
});
