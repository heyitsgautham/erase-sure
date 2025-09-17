import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useApp } from '../contexts/AppContext';
import type { Device, WipePlan, BackupResult } from '../contexts/AppContext';

export interface LogEvent {
    line: string;
    ts: string;
    stream: 'stdout' | 'stderr';
}

export interface RunResult {
    exitCode: number;
    stdout: string[];
    stderr: string[];
}

interface ExitEvent {
    code: number | null;
    ts: string;
}

const MAX_LOG_LINES = 2000;

export function useSecureWipe() {
    const { dispatch, addToast, addLog } = useApp();
    const [logs, setLogs] = useState<LogEvent[]>([]);
    const [running, setRunning] = useState(false);
    const [currentSession, setCurrentSession] = useState<string | null>(null);

    // Set up event listeners for streaming logs
    useEffect(() => {
        let unlistenStdout: UnlistenFn | undefined;
        let unlistenStderr: UnlistenFn | undefined;
        let unlistenExit: UnlistenFn | undefined;

        const setupListeners = async () => {
            unlistenStdout = await listen<LogEvent>('securewipe://stdout', (event) => {
                const logEvent: LogEvent = { ...event.payload, stream: 'stdout' };
                setLogs(prev => {
                    const newLogs = [...prev, logEvent];
                    return newLogs.slice(-MAX_LOG_LINES); // Keep only last MAX_LOG_LINES
                });
                // Only add to context logs, not duplicate
                addLog(`[${logEvent.ts}] ${logEvent.line}`);
            });

            unlistenStderr = await listen<LogEvent>('securewipe://stderr', (event) => {
                const logEvent: LogEvent = { ...event.payload, stream: 'stderr' };
                setLogs(prev => {
                    const newLogs = [...prev, logEvent];
                    return newLogs.slice(-MAX_LOG_LINES);
                });
                // Only add to context logs, not duplicate - filter out signing key errors
                if (!logEvent.line.includes('Failed to load signing key') && 
                    !logEvent.line.includes('SECUREWIPE_SIGN_KEY_PATH not set')) {
                    addLog(`[${logEvent.ts}] ERROR: ${logEvent.line}`);
                }
            });

            unlistenExit = await listen<ExitEvent>('securewipe://exit', (event) => {
                console.log('Process exited with code:', event.payload.code);
                setRunning(false);
                setCurrentSession(null);
                
                // Don't treat all non-zero exits as failures immediately
                // Let the specific operation handlers decide based on output content
            });
        };

        setupListeners();

        return () => {
            if (unlistenStdout) unlistenStdout();
            if (unlistenStderr) unlistenStderr();
            if (unlistenExit) unlistenExit();
        };
    }, [addLog]);

    const run = useCallback(async (args: string[], sessionId?: string): Promise<RunResult> => {
        const session = sessionId || `session_${Date.now()}`;
        setCurrentSession(session);
        setRunning(true);
        setLogs([]);
        dispatch({ type: 'CLEAR_LOGS' });

        try {
            await invoke('run_securewipe', { 
                args, 
                sessionId: session 
            });

            // Wait for the process to complete by listening for the exit event
            return new Promise((resolve, reject) => {
                let exitListener: UnlistenFn | undefined;

                const timeout = setTimeout(() => {
                    cleanup();
                    reject(new Error('Process timeout'));
                }, 1200000); // 20 minutes

                const cleanup = () => {
                    clearTimeout(timeout);
                    if (exitListener) exitListener();
                };

                // Set up exit listener only - reuse the global log listeners
                const setupListeners = async () => {
                    try {
                        // Collect logs from the existing listeners
                        const currentLogCapture: LogEvent[] = [];
                        
                        const logCapture = (event: any) => {
                            currentLogCapture.push(event.payload);
                        };

                        const unlisten1 = await listen<LogEvent>('securewipe://stdout', logCapture);
                        const unlisten2 = await listen<LogEvent>('securewipe://stderr', logCapture);

                        exitListener = await listen<ExitEvent>('securewipe://exit', (event) => {
                            unlisten1();
                            unlisten2();
                            cleanup();

                            const exitCode = event.payload.code ?? -1;
                            const stdout = currentLogCapture
                                .filter(log => log.stream === 'stdout')
                                .map(log => log.line);
                            const stderr = currentLogCapture
                                .filter(log => log.stream === 'stderr')
                                .map(log => log.line);

                            resolve({ exitCode, stdout, stderr });
                        });
                    } catch (error) {
                        cleanup();
                        reject(error);
                    }
                };

                setupListeners();
            });
        } catch (error) {
            setRunning(false);
            setCurrentSession(null);
            throw error;
        }
    }, [dispatch]);

    // Helper function to parse mixed NDJSON logs + JSON output format
    const parseJsonOutput = useCallback((lines: string[]): any => {
        // Filter out NDJSON log lines (they have "level" and "timestamp" fields)
        const nonLogLines: string[] = [];
        
        for (const line of lines) {
            if (!line.trim()) continue;
            
            try {
                const parsed = JSON.parse(line);
                // Skip if it looks like a log entry (has level and timestamp)
                if (parsed.level && parsed.timestamp) {
                    continue;
                }
                // If it's a valid JSON object/array that's not a log, keep it
                nonLogLines.push(line);
            } catch {
                // If it's not valid JSON, it might be part of a multi-line JSON
                nonLogLines.push(line);
            }
        }
        
        if (nonLogLines.length === 0) {
            throw new Error('No JSON output found in command output');
        }
        
        // Join all non-log lines and try to parse as a single JSON
        const jsonContent = nonLogLines.join('');
        try {
            return JSON.parse(jsonContent);
        } catch (error) {
            // If that fails, try parsing the last line that looked like valid JSON
            for (let i = nonLogLines.length - 1; i >= 0; i--) {
                try {
                    return JSON.parse(nonLogLines[i]);
                } catch {
                    continue;
                }
            }
            throw new Error(`Failed to parse JSON output: ${error}`);
        }
    }, []);

    const discover = useCallback(async (): Promise<Device[]> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Discovering devices...' });

        try {
            const result = await run(['discover', '--format', 'json']);

            if (result.exitCode === 0) {
                console.log('CLI stdout:', result.stdout);
                console.log('CLI stderr:', result.stderr);
                
                const devices: Device[] = parseJsonOutput(result.stdout);
                
                // Map the CLI output format to our expected Device interface
                const mappedDevices: Device[] = devices.map((device: any) => ({
                    path: device.name || device.path || '',
                    model: device.model || 'Unknown Device',
                    serial: device.serial || 'N/A',
                    capacity: device.capacity_bytes || device.capacity || 0,
                    bus: device.bus || 'Unknown',
                    mountpoints: device.mountpoints || [],
                    risk_level: device.risk_level || 'SAFE',
                    blocked: device.risk_level === 'CRITICAL',
                    block_reason: device.risk_level === 'CRITICAL' ? 'System disk with active mount points' : undefined
                }));
                
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
    }, [run, dispatch, addToast, parseJsonOutput]);

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
                const wipePlanData = parseJsonOutput(result.stdout);
                
                // Map the CLI output format to our expected WipePlan interface
                const wipePlan: WipePlan = {
                    device_path: wipePlanData.device_path || opts.device,
                    policy: wipePlanData.policy || 'PURGE',
                    main_method: wipePlanData.main_method || 'Unknown Method',
                    hpa_dco_clear: wipePlanData.hpa_dco_clear || false,
                    verification: {
                        samples: wipePlanData.verification?.samples || opts.samples || 128
                    },
                    blocked: wipePlanData.blocked || false,
                    block_reason: wipePlanData.block_reason
                };
                
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
        setLogs([]);
        dispatch({ type: 'CLEAR_LOGS' });
    }, [dispatch]);

    return {
        logs,
        running,
        run,
        discover,
        planWipe,
        backup,
        cancel,
        clearLogs
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


