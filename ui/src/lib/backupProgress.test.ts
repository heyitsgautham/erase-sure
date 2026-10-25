import { describe, expect, it } from 'vitest';
import {
    BACKUP_TOTAL_STEPS,
    COMPLETE_PROGRESS,
    failedProgress,
    friendlyBackupError,
    runningProgress,
    stepForLine,
} from './backupProgress';

describe('stepForLine', () => {
    it.each([
        ['Starting backup operation', 1],
        ['backup_dir_created', 2],
        ['Copying files', 3],
        ['manifest_created', 3],
        ['Verifying integrity', 4],
        ['certificate_created', 5],
    ])('maps %j to step %i', (line, step) => {
        expect(stepForLine(line)).toBe(step);
    });

    it('returns null for a line that names no step', () => {
        expect(stepForLine('Total bytes: 1024')).toBeNull();
    });

    it('lets the earliest step win when a line matches several', () => {
        expect(stepForLine('starting copy')).toBe(1);
    });
});

describe('progress values', () => {
    it('names the running step and its share of the bar', () => {
        expect(runningProgress(2)).toEqual({
            title: 'Encrypted Backup in Progress',
            currentStep: 2,
            totalSteps: BACKUP_TOTAL_STEPS,
            currentStepName: 'Setting up AES-256 encryption...',
            percentage: 40,
        });
    });

    it('fills the bar when done and empties it on failure', () => {
        expect(COMPLETE_PROGRESS).toMatchObject({ currentStep: 5, percentage: 100 });
        expect(failedProgress('disk full')).toMatchObject({
            title: 'Backup Failed',
            currentStep: 0,
            currentStepName: '❌ Error: disk full',
            percentage: 0,
        });
    });
});

describe('friendlyBackupError', () => {
    it.each([
        ['spawn securewipe failed', 'SecureWipe CLI not found'],
        ['permission denied on /backups', 'Permission denied. Please check'],
        ['No such file or directory', 'Source device or destination path not found'],
        ['Process timeout', 'Backup operation timed out'],
    ])('explains %j', (message, advice) => {
        expect(friendlyBackupError(new Error(message))).toContain(advice);
    });

    it('passes other messages through and covers non-errors', () => {
        expect(friendlyBackupError(new Error('Device busy'))).toBe('Device busy');
        expect(friendlyBackupError('boom')).toBe('Unknown error occurred');
    });
});
