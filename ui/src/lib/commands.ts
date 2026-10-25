/**
 * Typed wrappers around the desktop backend's Tauri commands.
 *
 * Each wrapper names the command once, passes its arguments, and parses the
 * result against a schema, so a backend change surfaces as a clear error
 * instead of a wrong value deep inside a screen.
 */
import { invoke } from '@tauri-apps/api/tauri';
import { z } from 'zod';
import { DirectoryListingSchema, WipeDeviceInfoSchema, describeIssues } from './schemas';

export type DirectoryListing = z.infer<typeof DirectoryListingSchema>;
export type FileSystemEntry = DirectoryListing['entries'][number];
export type WipeDeviceInfo = z.infer<typeof WipeDeviceInfoSchema>;

/** The backend answered, but not with what the command promises. */
export class CommandResponseError extends Error {
    constructor(
        readonly command: string,
        detail: string,
    ) {
        super(`Unexpected response from ${command}: ${detail}`);
        this.name = 'CommandResponseError';
    }
}

/** The page runs outside the desktop app, for example in a plain browser tab. */
export class DesktopUnavailableError extends Error {
    constructor() {
        super('The SecureWipe desktop backend is not running. Start the app with npm run tauri dev.');
        this.name = 'DesktopUnavailableError';
    }
}

export function hasDesktopBackend(): boolean {
    return typeof (window as { __TAURI_IPC__?: unknown }).__TAURI_IPC__ === 'function';
}

async function call<S extends z.ZodTypeAny>(command: string, args: Record<string, unknown>, schema: S): Promise<z.infer<S>> {
    if (!hasDesktopBackend()) {
        throw new DesktopUnavailableError();
    }
    const raw = await invoke<unknown>(command, args);
    const parsed = schema.safeParse(raw);
    if (!parsed.success) {
        throw new CommandResponseError(command, describeIssues(parsed.error));
    }
    return parsed.data;
}

const nothing = z.null().or(z.undefined());

export interface WipeConfirmation {
    device: string;
    serial: string;
    policy: string;
    user_input: string;
}

export const commands = {
    /** Spawn the CLI. Output and the exit code arrive as securewipe:// events. */
    runSecurewipe: (args: string[], sessionId: string) => call('run_securewipe', { args, sessionId }, nothing),
    cancelSecurewipe: (sessionId: string) => call('cancel_securewipe', { sessionId }, nothing),
    getHomeDir: () => call('get_home_dir', {}, z.string().min(1)),
    listCertFiles: (directory: string) => call('list_cert_files', { directory }, z.array(z.string())),
    readFileContent: (filePath: string) => call('read_file_content', { filePath }, z.string()),
    fileExists: (filePath: string) => call('file_exists', { filePath }, z.boolean()),
    openPath: (path: string) => call('open_path', { path }, nothing),
    /** Export a certificate to PDF and return where the PDF landed. */
    generatePdfForCert: (certJsonPath: string, sessionId: string) =>
        call('generate_pdf_for_cert', { certJsonPath, sessionId }, z.string().min(1)),
    /** List a folder; null lists the home folder. */
    browseFolders: (path: string | null) => call('browse_folders', { path }, DirectoryListingSchema),
    calculateSelectionSize: (paths: string[]) => call('calculate_selection_size', { paths }, z.number().nonnegative()),
    validateWipeDevice: (device: string) => call('validate_wipe_device', { device }, WipeDeviceInfoSchema),
    executeDestructiveWipe: (confirmation: WipeConfirmation, backupCertId: string | null) =>
        call('execute_destructive_wipe', { confirmation, backupCertId }, nothing),
};
