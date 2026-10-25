/**
 * Arguments for `securewipe backup` and how to read what it printed.
 *
 * The CLI signs the certificate after the backup itself finishes. A missing
 * signing key makes it exit non-zero even though the encrypted backup and its
 * unsigned certificate are on disk, so success is read from the output lines
 * rather than from the exit code.
 */

/** User folders backed up from a system disk when the user picks nothing else. */
export const DEFAULT_USER_DIRS = [
    '$HOME/Documents',
    '$HOME/Pictures',
    '$HOME/Videos',
    '$HOME/Music',
    '$HOME/Desktop',
    '$HOME/Downloads',
] as const;

export interface BackupOptions {
    device: string;
    dest: string;
    sign?: boolean;
    signKeyPath?: string;
    includePaths?: string[];
    /** The source is a system disk. The backend strips the flag after checking it. */
    allowCritical?: boolean;
}

export function buildBackupArgs(opts: BackupOptions): string[] {
    const args = ['backup', '--device', opts.device, '--dest', opts.dest];

    if (opts.includePaths && opts.includePaths.length > 0) {
        args.push('--paths', opts.includePaths.join(','));
    } else if (opts.allowCritical) {
        args.push('--paths', DEFAULT_USER_DIRS.join(','));
    }
    if (opts.sign) args.push('--sign');
    if (opts.signKeyPath) args.push('--sign-key-path', opts.signKeyPath);
    if (opts.allowCritical) args.push('--critical-ok');

    return args;
}

const SUCCESS_MARKERS = [
    'Backup completed successfully',
    'backup_complete',
    'Certificate saved to:',
    'Backup ID:',
    'Verification status: PASSED',
];

function isCriticalError(line: string): boolean {
    return (
        line.includes('Permission denied') ||
        line.includes('No such file') ||
        line.includes('Device busy') ||
        (line.includes('Error:') && !line.includes('Failed to load signing key'))
    );
}

export type BackupOutcome =
    { status: 'success' } | { status: 'failed'; message: string } | { status: 'unconfirmed'; message: string };

/** Decide whether a backup run worked from its stdout and stderr lines. */
export function classifyBackupOutput(stdout: string[], stderr: string[]): BackupOutcome {
    const errors = stderr.filter(isCriticalError);
    if (errors.length > 0) {
        return { status: 'failed', message: errors.join('\n') };
    }
    const succeeded = [...stdout, ...stderr].some((line) => SUCCESS_MARKERS.some((marker) => line.includes(marker)));
    if (succeeded) {
        return { status: 'success' };
    }
    return { status: 'unconfirmed', message: 'Backup process completed but success could not be confirmed' };
}

export interface BackupPaths {
    backupId?: string;
    certPathJson?: string;
    certPathPdf?: string;
    manifestSha256?: string;
}

const PATH_LABELS: [keyof BackupPaths, string][] = [
    ['backupId', 'Backup ID:'],
    ['certPathJson', 'Certificate JSON:'],
    ['certPathPdf', 'Certificate PDF:'],
    ['manifestSha256', 'Manifest SHA256:'],
];

/** Pull the backup ID and file paths out of `Label: value` lines. A later line wins. */
export function parseBackupPaths(lines: string[]): BackupPaths {
    const result: BackupPaths = {};
    for (const line of lines) {
        for (const [key, label] of PATH_LABELS) {
            const at = line.indexOf(label);
            if (at >= 0) {
                result[key] = line.slice(at + label.length).trim();
            }
        }
    }
    return result;
}
