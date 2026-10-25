import { describe, expect, it } from 'vitest';
import { buildBackupArgs, classifyBackupOutput, DEFAULT_USER_DIRS, parseBackupPaths } from './backupOutput';

describe('buildBackupArgs', () => {
    it('backs up a whole removable device with only the required flags', () => {
        expect(buildBackupArgs({ device: '/dev/sdb', dest: '/backups' })).toEqual([
            'backup', '--device', '/dev/sdb', '--dest', '/backups',
        ]);
    });

    it('limits a system disk backup to the user folders and marks it critical', () => {
        const args = buildBackupArgs({ device: '/dev/nvme0n1', dest: '/backups', allowCritical: true });

        expect(args).toEqual([
            'backup', '--device', '/dev/nvme0n1', '--dest', '/backups',
            '--paths', DEFAULT_USER_DIRS.join(','),
            '--critical-ok',
        ]);
    });

    it('prefers the paths the user picked and passes signing options', () => {
        const args = buildBackupArgs({
            device: '/dev/nvme0n1',
            dest: '/backups',
            includePaths: ['/home/me/a', '/home/me/b'],
            sign: true,
            signKeyPath: '/keys/dev.pem',
            allowCritical: true,
        });

        expect(args).toEqual([
            'backup', '--device', '/dev/nvme0n1', '--dest', '/backups',
            '--paths', '/home/me/a,/home/me/b',
            '--sign', '--sign-key-path', '/keys/dev.pem',
            '--critical-ok',
        ]);
    });

    it('treats an empty selection like no selection', () => {
        expect(buildBackupArgs({ device: '/dev/sdb', dest: '/b', includePaths: [] })).not.toContain('--paths');
    });
});

describe('classifyBackupOutput', () => {
    it('accepts a run that printed the completion line', () => {
        expect(classifyBackupOutput(['Backup completed successfully!', 'Backup ID: abc'], [])).toEqual({ status: 'success' });
    });

    it('accepts a finished backup whose certificate could not be signed', () => {
        const outcome = classifyBackupOutput(['Backup ID: abc'], ['Error: Failed to load signing key: missing']);

        expect(outcome).toEqual({ status: 'success' });
    });

    it('fails on permission, missing path, busy device and other errors, listing each one', () => {
        const stderr = ['Permission denied (os error 13)', 'note: retrying', 'Error: disk full'];

        expect(classifyBackupOutput(['Backup ID: abc'], stderr)).toEqual({
            status: 'failed',
            message: 'Permission denied (os error 13)\nError: disk full',
        });
        expect(classifyBackupOutput([], ['Device busy']).status).toBe('failed');
        expect(classifyBackupOutput([], ['No such file or directory']).status).toBe('failed');
    });

    it('refuses to call a silent run a success', () => {
        expect(classifyBackupOutput(['Starting backup'], [])).toEqual({
            status: 'unconfirmed',
            message: 'Backup process completed but success could not be confirmed',
        });
    });
});

describe('parseBackupPaths', () => {
    it('reads the backup ID and file paths from labelled lines', () => {
        const paths = parseBackupPaths([
            'Backup ID: 1234',
            'Certificate JSON: /home/me/SecureWipe/certificates/1234.json',
            'Certificate PDF: /home/me/SecureWipe/certificates/1234.pdf',
            'Manifest SHA256: deadbeef',
        ]);

        expect(paths).toEqual({
            backupId: '1234',
            certPathJson: '/home/me/SecureWipe/certificates/1234.json',
            certPathPdf: '/home/me/SecureWipe/certificates/1234.pdf',
            manifestSha256: 'deadbeef',
        });
    });

    it('returns nothing for output without labels', () => {
        expect(parseBackupPaths(['Starting backup'])).toEqual({});
    });
});
