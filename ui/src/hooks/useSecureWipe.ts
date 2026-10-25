import { useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/api/shell';
import { useApp } from '../contexts/AppContext';
import type { Device, WipePlan, BackupResult } from '../contexts/AppContext';
import { parseJsonOutput, toDevice, toWipePlan } from '../lib/cliOutput';
import { CLI_EVENTS } from '../types/securewipe';
import type { ExitEvent, LogEvent, RunResult, VerifyResult } from '../types/securewipe';

/** The desktop backend kills the CLI after 20 minutes, so the UI stops waiting then too. */
const RUN_TIMEOUT_MS = 20 * 60 * 1000;
import type { CliDevice, CliWipePlan } from '../lib/cliOutput';

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
                console.log('CLI stdout:', result.stdout);
                console.log('CLI stderr:', result.stderr);
                
                const devices = parseJsonOutput<CliDevice[]>(result.stdout);
                
                // Map the CLI output format to our expected Device interface
                const mappedDevices: Device[] = devices.map(toDevice);
                
                dispatch({ type: 'SET_DEVICES', payload: mappedDevices });
                addToast(`Discovered ${mappedDevices.length} devices`, 'success');
                
                return mappedDevices;
            } else {
                const errorMsg = result.stderr.join('\n') || 'Failed to discover devices';
                console.error('Device discovery failed:', errorMsg);
                throw new Error(errorMsg);
            }
        } catch (error) {
            console.error('Discover error:', error);
            const errorMessage = error instanceof Error ? error.message : 'Unknown error during device discovery';
            addToast(errorMessage, 'error');
            dispatch({ type: 'SET_DEVICES', payload: [] }); // Clear devices on error
            throw error;
        } finally {
            dispatch({ type: 'SET_OPERATION', payload: null });
        }
    }, [run, dispatch, addToast]);

    const planWipe = useCallback(async (opts: {
        device: string;
        samples?: number;
        isoMode?: boolean;
        noEnrich?: boolean;
    }): Promise<WipePlan> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Creating wipe plan...' });

        try {
            const args = ['wipe', '--device', opts.device, '--format', 'json'];
            if (opts.samples) args.push('--samples', String(opts.samples));
            if (opts.isoMode) args.push('--iso-mode');
            if (opts.noEnrich) args.push('--no-enrich');

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

    const backup = useCallback(async (opts: {
        device: string;
        dest: string;
        sign?: boolean;
        signKeyPath?: string;
        includePaths?: string[];
        allowCritical?: boolean;
    }): Promise<{ certPathJson?: string; certPathPdf?: string; manifestSha256?: string }> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Running backup...' });

        // Progress will be tracked through the existing log listener in useEffect
        // The backup component will update progress based on log patterns

        try {
            const args = ['backup', '--device', opts.device, '--dest', opts.dest];
            
            // Default to common user directories if no paths specified and critical disk
            if (!opts.includePaths || opts.includePaths.length === 0) {
                if (opts.allowCritical) {
                    // Default user directories for system disk backup
                    const defaultPaths = [
                        '$HOME/Documents',
                        '$HOME/Pictures', 
                        '$HOME/Videos',
                        '$HOME/Music',
                        '$HOME/Desktop',
                        '$HOME/Downloads'
                    ];
                    args.push('--paths', defaultPaths.join(','));
                }
            } else {
                args.push('--paths', opts.includePaths.join(','));
            }
            
            if (opts.sign) {
                args.push('--sign');
            }
            if (opts.signKeyPath) {
                args.push('--sign-key-path', opts.signKeyPath);
            }
            if (opts.allowCritical) {
                args.push('--critical-ok'); // This will be filtered out by backend
            }

            const result = await run(args);

            // Check if backup actually succeeded by looking for success indicators in logs
            const allOutput = [...result.stdout, ...result.stderr];
            const hasSuccessLog = allOutput.some(line => 
                line.includes('Backup completed successfully') || 
                line.includes('backup_complete') ||
                line.includes('Certificate saved to:') ||
                line.includes('Backup ID:') ||
                line.includes('Verification status: PASSED')
            );

            // Check for critical errors (not signing key errors)
            const hasCriticalError = result.stderr.some(line => 
                line.includes('Permission denied') ||
                line.includes('No such file') ||
                line.includes('Device busy') ||
                (line.includes('Error:') && !line.includes('Failed to load signing key'))
            );

            // Backup succeeded if we see success indicators and no critical errors
            if (hasSuccessLog && !hasCriticalError) {
                // Parse certificate paths from output
                const certPaths = parseBackupPaths(allOutput);
                
                // Extract backup ID from logs
                const backupIdLine = allOutput.find(line => line.includes('Backup ID:'));
                const backupId = backupIdLine ? backupIdLine.split('Backup ID:')[1]?.trim() : 'unknown';
                
                const backupResult: BackupResult = {
                    backup_path: `${opts.dest}/${backupId}`,
                    certificate_json_path: certPaths.certPathJson,
                    certificate_pdf_path: certPaths.certPathPdf,
                    manifest_path: `${opts.dest}/${backupId}/manifest.json`,
                    integrity_checks: 5 // This should be parsed from output
                };

                dispatch({ type: 'SET_BACKUP_RESULT', payload: backupResult });
                addToast('Backup completed successfully', 'success');
                return certPaths;
            } else if (hasCriticalError) {
                // Only throw error if we have critical errors
                const errorLines = result.stderr.filter(line => 
                    line.includes('Permission denied') ||
                    line.includes('No such file') ||
                    line.includes('Device busy') ||
                    (line.includes('Error:') && !line.includes('Failed to load signing key'))
                );
                const errorMsg = errorLines.length > 0 ? errorLines.join('\n') : 'Backup process failed';
                throw new Error(errorMsg);
            } else {
                // If no success indicators and no critical errors, it might be a timeout or other issue
                throw new Error('Backup process completed but success could not be confirmed');
            }
        } finally {
            dispatch({ type: 'SET_OPERATION', payload: null });
        }
    }, [run, dispatch, addToast]);

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
            const errorMessage = error instanceof Error ? error.message : 'Failed to open file';
            addToast(errorMessage, 'error');
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
            console.warn('Direct verification failed, falling back to portal UI:', error);
            
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

// Helper function to parse backup certificate paths from CLI output
function parseBackupPaths(stdout: string[]): {
    certPathJson?: string;
    certPathPdf?: string;
    manifestSha256?: string;
} {
    const result: {
        certPathJson?: string;
        certPathPdf?: string;
        manifestSha256?: string;
    } = {};

    for (const line of stdout) {
        if (line.includes('Certificate JSON:')) {
            result.certPathJson = line.split('Certificate JSON:')[1]?.trim();
        } else if (line.includes('Certificate PDF:')) {
            result.certPathPdf = line.split('Certificate PDF:')[1]?.trim();
        } else if (line.includes('Manifest SHA256:')) {
            result.manifestSha256 = line.split('Manifest SHA256:')[1]?.trim();
        }
    }

    return result;
}


