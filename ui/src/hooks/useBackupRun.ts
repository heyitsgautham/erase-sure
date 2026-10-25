import { useCallback, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import type { Device } from '../contexts/AppContext';
import { useSecureWipe } from './useSecureWipe';
import {
    COMPLETE_PROGRESS,
    LAST_TIMER_STEP,
    failedProgress,
    friendlyBackupError,
    runningProgress,
    stepForLine,
} from '../lib/backupProgress';
import { createLogger } from '../lib/logger';

const log = createLogger('backup');

/** How often the bar moves forward on its own while the CLI is quiet. */
const FALLBACK_STEP_MS = 8000;
/** How long the finished bar stays up before the certificates open. */
const SHOW_COMPLETE_MS = 4000;
/** How long a failure stays up before the form is ready for a retry. */
const SHOW_FAILURE_MS = 10000;

export interface BackupRequest {
    device: Device;
    destination: string;
    signKeyPath?: string;
    includePaths?: string[];
}

/**
 * Run one backup and drive the shared progress bar for it.
 *
 * The bar follows the CLI's output, a timer nudges it along when output goes
 * quiet, and the backup result alone decides whether it ends finished or
 * failed. Timers and follow-ups stop when the screen unmounts.
 */
export function useBackupRun() {
    const navigate = useNavigate();
    const { state, addToast, dispatch, clearLogs } = useApp();
    const { backup, running } = useSecureWipe();

    // Refs, not state: the run's handlers were created before the timer started.
    const progressTimer = useRef<ReturnType<typeof setInterval> | null>(null);
    const reachedStep = useRef(0);
    const pending = useRef<ReturnType<typeof setTimeout>[]>([]);

    const stopProgressTimer = useCallback(() => {
        if (progressTimer.current) {
            clearInterval(progressTimer.current);
            progressTimer.current = null;
        }
    }, []);

    const schedule = useCallback((fn: () => void, ms: number) => {
        pending.current.push(setTimeout(fn, ms));
    }, []);

    // Only the Backup screen shows the bar, so leaving it clears the bar too.
    useEffect(() => {
        const timers = pending.current;
        return () => {
            stopProgressTimer();
            timers.forEach(clearTimeout);
            dispatch({ type: 'SET_PROGRESS', payload: null });
        };
    }, [dispatch, stopProgressTimer]);

    // Start with an empty log so output from device discovery does not read as backup progress.
    useEffect(() => {
        clearLogs();
    }, [clearLogs]);

    // While the backup runs, move the bar to the step the newest output line names.
    useEffect(() => {
        if (!running) return;
        const latest = state.logs[state.logs.length - 1];
        const step = latest ? stepForLine(latest.line) : null;
        if (step) {
            reachedStep.current = Math.max(reachedStep.current, step);
            dispatch({ type: 'SET_PROGRESS', payload: runningProgress(step) });
        }
    }, [state.logs, running, dispatch]);

    const startBackup = useCallback(
        async ({ device, destination, signKeyPath, includePaths }: BackupRequest) => {
            if (!destination.trim()) {
                addToast('Please specify a backup destination', 'error');
                return;
            }

            dispatch({ type: 'CLEAR_LOGS' });
            addToast(`Starting backup of ${device.model}...`, 'info');

            reachedStep.current = 1;
            dispatch({ type: 'SET_PROGRESS', payload: runningProgress(1) });

            stopProgressTimer();
            progressTimer.current = setInterval(() => {
                if (reachedStep.current < LAST_TIMER_STEP) {
                    reachedStep.current += 1;
                    dispatch({ type: 'SET_PROGRESS', payload: runningProgress(reachedStep.current) });
                }
            }, FALLBACK_STEP_MS);

            try {
                const result = await backup({
                    device: device.path,
                    dest: destination,
                    sign: true,
                    signKeyPath: signKeyPath || undefined,
                    includePaths,
                    allowCritical: device.risk_level === 'CRITICAL',
                });
                stopProgressTimer();

                dispatch({ type: 'SET_PROGRESS', payload: COMPLETE_PROGRESS });
                let completionMessage = 'Backup completed successfully! 🎉';
                if (result.certPathJson) {
                    completionMessage += ` Certificate saved to: ${result.certPathJson}`;
                }
                addToast(completionMessage, 'success');

                schedule(() => {
                    dispatch({ type: 'SET_PROGRESS', payload: null });
                    navigate('/certificates');
                }, SHOW_COMPLETE_MS);
            } catch (error) {
                stopProgressTimer();
                log.error('backup failed', { device: device.path }, error);

                const errorMessage = friendlyBackupError(error);
                dispatch({ type: 'SET_PROGRESS', payload: failedProgress(errorMessage) });
                addToast(`Backup operation failed: ${errorMessage}`, 'error');

                schedule(() => {
                    dispatch({ type: 'SET_PROGRESS', payload: null });
                }, SHOW_FAILURE_MS);
            }
        },
        [addToast, backup, dispatch, navigate, schedule, stopProgressTimer],
    );

    return { running, startBackup };
}
