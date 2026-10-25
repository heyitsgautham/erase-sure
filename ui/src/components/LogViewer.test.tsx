import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import LogViewer from './LogViewer';
import type { LogEvent } from '../types/securewipe';

const logs: LogEvent[] = [
    { line: 'Copying files', ts: '2026-01-01T00:00:00Z', stream: 'stdout' },
    { line: 'Permission denied', ts: '2026-01-01T00:00:01Z', stream: 'stderr' },
];

describe('LogViewer', () => {
    it('says so when there are no logs and disables copying', () => {
        render(<LogViewer logs={[]} />);

        expect(screen.getByText('No logs yet...')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Copy Logs' })).toBeDisabled();
    });

    it('labels each line with its stream', () => {
        render(<LogViewer logs={logs} title="Backup Progress" />);

        expect(screen.getByText('Backup Progress')).toBeInTheDocument();
        expect(screen.getByText('Copying files').parentElement).toHaveClass('log-stdout');
        expect(screen.getByText('Permission denied').parentElement).toHaveClass('log-stderr');
        expect(screen.getByText('STDERR:')).toHaveClass('log-stream-stderr');
    });

    it('copies every line with its timestamp and stream', async () => {
        const user = userEvent.setup();
        const writeText = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
        render(<LogViewer logs={logs} />);

        await user.click(screen.getByRole('button', { name: 'Copy Logs' }));

        expect(writeText).toHaveBeenCalledWith(
            '[2026-01-01T00:00:00Z] STDOUT: Copying files\n[2026-01-01T00:00:01Z] STDERR: Permission denied',
        );
    });
});
