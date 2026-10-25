import { useCallback, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/api/shell';
import { useApp } from '../contexts/AppContext';
import type { Device, WipePlan, BackupResult } from '../contexts/AppContext';
import { z } from 'zod';
import { buildWipePlanArgs, parseCliOutput, toDevice, toWipePlan } from '../lib/cliOutput';
import { commands } from '../lib/commands';
import { CliDeviceSchema, CliWipePlanSchema, VerifyResultSchema, describeIssues } from '../lib/schemas';
import { PORTAL_URL } from '../lib/config';
import { verifyUrl } from '../lib/certificates';
import { buildBackupArgs, classifyBackupOutput, parseBackupPaths } from '../lib/backupOutput';
import type { BackupOptions, BackupPaths } from '../lib/backupOutput';
import { CLI_EVENTS } from '../types/securewipe';
import type { ExitEvent, LogEvent, RunResult, VerifyResult } from '../types/securewipe';

/** A verified result from the portal, or a note that the portal page was opened instead. */
export type VerifyOutcome = { kind: 'verified'; result: VerifyResult } | { kind: 'opened-portal' };

/** The desktop backend kills the CLI after 20 minutes, so the UI stops waiting then too. */
const RUN_TIMEOUT_MS = 20 * 60 * 1000;
import type { WipePlanOptions } from '../lib/cliOutput';
import { createLogger, describeError } from '../lib/logger';

const log = createLogger('securewipe');

export function useSecureWipe() {
    const { dispatch, addToast } = useApp();
    const [running, setRunning] = useState(false);
    const [currentSession, setCurrentSession] = useState<string | null>(null);

    const run = useCallback(
        async (args: string[], sessionId?: string): Promise<RunResult> => {
            const session = sessionId || `session_${Date.now()}`;
            setCurrentSession(session);
            setRunning(true);
            dispatch({ type: 'CLEAR_LOGS' });

            const stdout: string[] = [];
            const stderr: string[] = [];
            const unlisteners: UnlistenFn[] = [];
            let timer: ReturnType<typeof setTimeout> | undefined;
            let resolveExit: (code: number) => void = () => {};
            const exited = new Promise<number>((resolve) => {
                resolveExit = resolve;
            });

            try {
                // Subscribe before spawning. The backend streams output on its own thread, so a fast
                // CLI can print and exit before run_securewipe has even returned.
                unlisteners.push(await listen<LogEvent>(CLI_EVENTS.stdout, (event) => stdout.push(event.payload.line)));
                unlisteners.push(await listen<LogEvent>(CLI_EVENTS.stderr, (event) => stderr.push(event.payload.line)));
                unlisteners.push(
                    await listen<ExitEvent>(CLI_EVENTS.exit, (event) => resolveExit(event.payload.code ?? -1)),
                );

                await commands.runSecurewipe(args, session);

                const timedOut = new Promise<never>((_, reject) => {
                    timer = setTimeout(() => reject(new Error('Process timeout')), RUN_TIMEOUT_MS);
                });
                const exitCode = await Promise.race([exited, timedOut]);
                return { exitCode, stdout, stderr };
            } finally {
                clearTimeout(timer);
                unlisteners.forEach((unlisten) => unlisten());
                setRunning(false);
                setCurrentSession(null);
            }
        },
        [dispatch],
    );

    const discover = useCallback(async (): Promise<Device[]> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Discovering devices...' });

        try {
            const result = await run(['discover', '--format', 'json']);

            if (result.exitCode === 0) {
                const devices = parseCliOutput(result.stdout, z.array(CliDeviceSchema), 'device list');

                // Map the CLI output format to our expected Device interface
                const mappedDevices: Device[] = devices.map(toDevice);

                dispatch({ type: 'SET_DEVICES', payload: mappedDevices });
                addToast(`Discovered ${mappedDevices.length} devices`, 'success');

                return mappedDevices;
            } else {
                const errorMsg = result.stderr.join('\n') || 'Failed to discover devices';
                throw new Error(errorMsg);
            }
        } catch (error) {
            log.error('device discovery failed', undefined, error);
            const errorMessage = error instanceof Error ? error.message : 'Unknown error during device discovery';
            addToast(errorMessage, 'error');
            dispatch({ type: 'SET_DEVICES', payload: [] }); // Clear devices on error
            throw error;
        } finally {
            dispatch({ type: 'SET_OPERATION', payload: null });
        }
    }, [run, dispatch, addToast]);

    const planWipe = useCallback(
        async (opts: WipePlanOptions): Promise<WipePlan> => {
            dispatch({ type: 'SET_OPERATION', payload: 'Creating wipe plan...' });

            try {
                const args = buildWipePlanArgs(opts);
                const result = await run(args);

                if (result.exitCode === 0) {
                    const wipePlanData = parseCliOutput(result.stdout, CliWipePlanSchema, 'wipe plan');

                    // Map the CLI output format to our expected WipePlan interface
                    const wipePlan: WipePlan = toWipePlan(wipePlanData, opts.device, opts.samples);

                    dispatch({ type: 'SET_WIPE_PLAN', payload: wipePlan });
                    addToast('Wipe plan created successfully', 'success');
                    return wipePlan;
                } else {
                    throw new Error(result.stderr.join('\n') || 'Failed to create wipe plan');
                }
            } finally {
                dispatch({ type: 'SET_OPERATION', payload: null });
            }
        },
        [run, dispatch, addToast],
    );

    const backup = useCallback(
        async (opts: BackupOptions): Promise<BackupPaths> => {
            dispatch({ type: 'SET_OPERATION', payload: 'Running backup...' });

            try {
                const result = await run(buildBackupArgs(opts));
                const outcome = classifyBackupOutput(result.stdout, result.stderr);
                if (outcome.status !== 'success') {
                    throw new Error(outcome.message);
                }

                const paths = parseBackupPaths([...result.stdout, ...result.stderr]);
                const backupId = paths.backupId ?? 'unknown';
                const backupResult: BackupResult = {
                    backup_path: `${opts.dest}/${backupId}`,
                    certificate_json_path: paths.certPathJson,
                    certificate_pdf_path: paths.certPathPdf,
                    manifest_path: `${opts.dest}/${backupId}/manifest.json`,
                    integrity_checks: 5,
                };

                dispatch({ type: 'SET_BACKUP_RESULT', payload: backupResult });
                return paths;
            } finally {
                dispatch({ type: 'SET_OPERATION', payload: null });
            }
        },
        [run, dispatch],
    );

    const cancel = useCallback(async (): Promise<void> => {
        if (currentSession) {
            try {
                await commands.cancelSecurewipe(currentSession);
                addToast('Process cancelled', 'warning');
            } catch (error) {
                log.warn('cancel failed', { session: currentSession }, error);
                addToast('Failed to cancel process', 'error');
            }
        }
    }, [currentSession, addToast]);

    const clearLogs = useCallback(() => {
        dispatch({ type: 'CLEAR_LOGS' });
    }, [dispatch]);

    const generatePdfForCert = useCallback(
        async (certJsonPath: string): Promise<{ pdfPath: string }> => {
            dispatch({ type: 'SET_OPERATION', payload: 'Generating PDF certificate...' });

            try {
                const session = `pdf_session_${Date.now()}`;
                setCurrentSession(session);
                setRunning(true);

                // Call the updated Tauri command that returns the PDF path directly
                const pdfPath = await commands.generatePdfForCert(certJsonPath, session);

                setRunning(false);
                setCurrentSession(null);

                return { pdfPath };
            } catch (error) {
                setRunning(false);
                setCurrentSession(null);
                dispatch({ type: 'SET_OPERATION', payload: null });
                throw error;
            } finally {
                dispatch({ type: 'SET_OPERATION', payload: null });
            }
        },
        [dispatch],
    );

    const openPath = useCallback(
        async (path: string): Promise<void> => {
            try {
                await commands.openPath(path);
                addToast(`Opened ${path.split('/').pop()}`, 'success');
            } catch (error) {
                // Tauri commands reject with the backend's message as a plain string.
                addToast(describeError(error).message || 'Failed to open file', 'error');
                throw error;
            }
        },
        [addToast],
    );

    const verifyOnline = useCallback(async (certJson: Record<string, unknown>): Promise<VerifyOutcome> => {
        try {
            const response = await fetch(`${PORTAL_URL}/verify`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(certJson),
            });
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
            const parsed = VerifyResultSchema.safeParse(await response.json());
            if (!parsed.success) {
                throw new Error(`Unexpected verify response: ${describeIssues(parsed.error)}`);
            }
            return { kind: 'verified', result: parsed.data };
        } catch (error) {
            // Without a direct answer, let the user check the certificate in the portal itself.
            log.warn('direct verification failed, opening the portal instead', undefined, error);
            const certId = typeof certJson.cert_id === 'string' ? certJson.cert_id : undefined;
            await open(certId ? verifyUrl(PORTAL_URL, certId) : PORTAL_URL);
            return { kind: 'opened-portal' };
        }
    }, []);

    return {
        running,
        run,
        discover,
        planWipe,
        backup,
        cancel,
        clearLogs,
        generatePdfForCert,
        openPath,
        verifyOnline,
    };
}
