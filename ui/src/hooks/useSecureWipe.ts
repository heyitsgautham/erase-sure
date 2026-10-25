import { useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/api/shell';
import { useApp } from '../contexts/AppContext';
import type { Device, WipePlan, BackupResult } from '../contexts/AppContext';
import { buildWipePlanArgs, parseJsonOutput, toDevice, toWipePlan } from '../lib/cliOutput';
import { buildBackupArgs, classifyBackupOutput, parseBackupPaths } from '../lib/backupOutput';
import type { BackupOptions, BackupPaths } from '../lib/backupOutput';
import { CLI_EVENTS } from '../types/securewipe';
import type { ExitEvent, LogEvent, RunResult, VerifyResult } from '../types/securewipe';

/** The desktop backend kills the CLI after 20 minutes, so the UI stops waiting then too. */
const RUN_TIMEOUT_MS = 20 * 60 * 1000;
import type { CliDevice, CliWipePlan, WipePlanOptions } from '../lib/cliOutput';
import { createLogger, describeError } from '../lib/logger';

const log = createLogger('securewipe');

export function useSecureWipe() {
    const { dispatch, addToast } = useApp();
    const [running, setRunning] = useState(false);
    const [currentSession, setCurrentSession] = useState<string | null>(null);

    const run = useCallback(async (args: string[], sessionId?: string): Promise<RunResult> => {
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
            unlisteners.push(await listen<ExitEvent>(CLI_EVENTS.exit, (event) => resolveExit(event.payload.code ?? -1)));

            await invoke('run_securewipe', { args, sessionId: session });

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
    }, [dispatch]);

    const discover = useCallback(async (): Promise<Device[]> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Discovering devices...' });

        try {
            const result = await run(['discover', '--format', 'json']);

            if (result.exitCode === 0) {
                
                const devices = parseJsonOutput<CliDevice[]>(result.stdout);
                
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

    const planWipe = useCallback(async (opts: WipePlanOptions): Promise<WipePlan> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Creating wipe plan...' });

        try {
            const args = buildWipePlanArgs(opts);
            const result = await run(args);

            if (result.exitCode === 0) {
                const wipePlanData = parseJsonOutput<CliWipePlan>(result.stdout);
                
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
    }, [run, dispatch, addToast]);

    const backup = useCallback(async (opts: BackupOptions): Promise<BackupPaths> => {
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
    }, [run, dispatch]);

    const cancel = useCallback(async (): Promise<void> => {
        if (currentSession) {
            try {
                await invoke('cancel_securewipe', { sessionId: currentSession });
                addToast('Process cancelled', 'warning');
            } catch (error) {
                addToast('Failed to cancel process', 'error');
            }
        }
    }, [currentSession, addToast]);

    const clearLogs = useCallback(() => {
        dispatch({ type: 'CLEAR_LOGS' });
    }, [dispatch]);

    const generatePdfForCert = useCallback(async (certJsonPath: string): Promise<{ pdfPath: string }> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Generating PDF certificate...' });

        try {
            const session = `pdf_session_${Date.now()}`;
            setCurrentSession(session);
            setRunning(true);

            // Call the updated Tauri command that returns the PDF path directly
            const pdfPath = await invoke('generate_pdf_for_cert', { 
                certJsonPath, 
                sessionId: session 
            }) as string;

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
    }, [dispatch]);

    const openPath = useCallback(async (path: string): Promise<void> => {
        try {
            await invoke('open_path', { path });
            addToast(`Opened ${path.split('/').pop()}`, 'success');
        } catch (error) {
            // Tauri commands reject with the backend's message as a plain string.
            addToast(describeError(error).message || 'Failed to open file', 'error');
            throw error;
        }
    }, [addToast]);

    const verifyOnline = useCallback(async (certJson: object): Promise<VerifyResult> => {
        try {
            const PORTAL_URL = 'http://localhost:8000'; // TODO: Make configurable
            
            const response = await fetch(`${PORTAL_URL}/verify`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(certJson),
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const result = await response.json();
            return result;

        } catch (error) {
            log.warn('direct verification failed, opening the portal instead', undefined, error);
            
            // Fallback: open portal in browser with cert_id
            const { cert_id: certId, cert_type: certType } = certJson as { cert_id?: string; cert_type?: string };
            if (certId) {
                const portalUrl = `http://localhost:8000/verify?cert_id=${encodeURIComponent(certId)}`;
                await open(portalUrl);
            } else {
                await open('http://localhost:8000');
            }
            
            // Return a fallback result indicating manual verification is needed
            return {
                schema_valid: null,
                signature_valid: null,
                hash_valid: null,
                chain_valid: null,
                cert_summary: {
                    cert_id: certId || 'unknown',
                    cert_type: certType || 'unknown',
                    device_model: 'unknown'
                },
                computed: {},
                errors: ['Direct verification failed. Please verify manually in the opened portal page.'],
                fallback: true
            } as VerifyResult & { fallback: true };
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
        verifyOnline
    };
}
