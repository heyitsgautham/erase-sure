/**
 * What the Backup screen's progress bar shows.
 *
 * The CLI does not report progress, so the bar follows keywords in its output
 * lines. A slow timer moves it forward between lines, never past step 4,
 * because only a finished run may show the last step as done.
 */
import type { ProgressInfo } from '../contexts/AppContext';

export const BACKUP_STEP_NAMES = [
    'Initializing backup process...',
    'Setting up AES-256 encryption...',
    'Copying and encrypting files...',
    'Verifying backup integrity...',
    'Generating certificates...',
] as const;

export const BACKUP_TOTAL_STEPS = BACKUP_STEP_NAMES.length;

/** The fallback timer stops here; step 5 waits for the run to finish. */
export const LAST_TIMER_STEP = 4;

const STEP_PATTERNS: [RegExp, number][] = [
    [/starting|initializing|begin|backup_start/i, 1],
    [/encryption|aes|key|crypto|cipher|backup_dir_created/i, 2],
    [/copying|copy|encrypt|reading|writing|files|file_processing|manifest_created/i, 3],
    [/verifying|verify|check|integrity|hash|sample|verification_complete/i, 4],
    [/certificate_created|backup_complete|sign|complete|finished|done|success/i, 5],
];

/** The step a CLI output line points to, or null when it names none. The first match wins. */
export function stepForLine(line: string): number | null {
    for (const [pattern, step] of STEP_PATTERNS) {
        if (pattern.test(line)) return step;
    }
    return null;
}

export function runningProgress(step: number): ProgressInfo {
    return {
        title: 'Encrypted Backup in Progress',
        currentStep: step,
        totalSteps: BACKUP_TOTAL_STEPS,
        currentStepName: BACKUP_STEP_NAMES[step - 1],
        percentage: (step / BACKUP_TOTAL_STEPS) * 100,
    };
}

export const COMPLETE_PROGRESS: ProgressInfo = {
    title: 'Backup Complete!',
    currentStep: BACKUP_TOTAL_STEPS,
    totalSteps: BACKUP_TOTAL_STEPS,
    currentStepName: '✅ All operations completed successfully',
    percentage: 100,
};

export function failedProgress(message: string): ProgressInfo {
    return {
        title: 'Backup Failed',
        currentStep: 0,
        totalSteps: BACKUP_TOTAL_STEPS,
        currentStepName: `❌ Error: ${message}`,
        percentage: 0,
    };
}

/** Turn a backup failure into advice the user can act on. */
export function friendlyBackupError(error: unknown): string {
    if (!(error instanceof Error)) return 'Unknown error occurred';
    const message = error.message;
    if (message.includes('securewipe')) {
        return 'SecureWipe CLI not found. Please ensure the core backup tool is built and accessible.';
    }
    if (message.includes('permission')) {
        return 'Permission denied. Please check file/folder permissions for the backup destination.';
    }
    if (message.includes('No such file')) {
        return 'Source device or destination path not found. Please verify the paths exist.';
    }
    if (message.includes('timeout')) {
        return 'Backup operation timed out. This may happen with very large files. Please try with smaller folders or check system resources.';
    }
    return message;
}
