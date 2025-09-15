import { useCallback, useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useApp } from '../contexts/AppContext';
import type {
    LogEvent,
    ExitEvent,
    RunResult,
    Device,
    WipePlan,
    BackupResult,
    BackupOptions,
    WipePlanOptions,
    DiscoverOptions
} from '../types/securewipe';

export function useSecureWipe() {
    const { dispatch, addToast, addLog } = useApp();
    const [logs, setLogs] = useState<LogEvent[]>([]);
    const [running, setRunning] = useState(false);
    const unlistenersRef = useRef<UnlistenFn[]>([]);

    // Clean up event listeners on unmount
    useEffect(() => {
        return () => {
            unlistenersRef.current.forEach(unlisten => unlisten());
        };
    }, []);

    const clearLogs = useCallback(() => {
        setLogs([]);
    }, []);

    const run = useCallback(async (args: string[]): Promise<RunResult> => {
        setRunning(true);
        setLogs([]);

        // Clean up existing listeners
        unlistenersRef.current.forEach(unlisten => unlisten());
        unlistenersRef.current = [];

        const stdout: string[] = [];
        const stderr: string[] = [];

        return new Promise(async (resolve, reject) => {
            try {
                // Set up event listeners
                const stdoutUnlisten = await listen<LogEvent>('securewipe://stdout', (event) => {
                    const logEvent = event.payload;
                    console.log('📥 Received stdout:', logEvent.line);
                    setLogs(prev => [...prev.slice(-1999), logEvent]); // Keep last 2000 lines
                    stdout.push(logEvent.line);
                    addLog(logEvent.line);
                });

                const stderrUnlisten = await listen<LogEvent>('securewipe://stderr', (event) => {
                    const logEvent = event.payload;
                    console.log('📥 Received stderr:', logEvent.line);
                    setLogs(prev => [...prev.slice(-1999), logEvent]); // Keep last 2000 lines
                    stderr.push(logEvent.line);
                    addLog(`[STDERR] ${logEvent.line}`);
                });

                const exitUnlisten = await listen<ExitEvent>('securewipe://exit', (event) => {
                    const exitEvent = event.payload;
                    console.log('🏁 Process exit:', exitEvent);
                    console.log('📋 Final stdout collected:', stdout);
                    console.log('📋 Final stderr collected:', stderr);
                    setRunning(false);

                    if (exitEvent.code === 0) {
                        addToast('Command completed successfully', 'success');
                    } else {
                        addToast(`Command failed with exit code: ${exitEvent.code}`, 'error');
                    }

                    resolve({
                        exitCode: exitEvent.code,
                        stdout,
                        stderr,
                    });
                });

                unlistenersRef.current = [stdoutUnlisten, stderrUnlisten, exitUnlisten];

                // Start the process
                await invoke('run_securewipe', { args });

            } catch (error) {
                setRunning(false);
                console.error('Invoke error:', error);
                let errorMessage = 'Unknown error';

                if (error instanceof Error) {
                    errorMessage = error.message;
                } else if (typeof error === 'string') {
                    errorMessage = error;
                } else if (error && typeof error === 'object') {
                    errorMessage = JSON.stringify(error);
                }

                addToast(`Error: ${errorMessage}`, 'error');
                reject(new Error(errorMessage));
            }
        });
    }, [addToast, addLog]);

    const discover = useCallback(async (options: DiscoverOptions = {}): Promise<Device[]> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Discovering devices...' });

        try {
            const args = ['discover', '--format', options.format || 'json'];
            if (options.noEnrich) {
                args.push('--no-enrich');
            }

            const result = await run(args);
            console.log('🔍 Discover - Full result:', result);
            console.log('🔍 Discover - Exit code:', result.exitCode);
            console.log('🔍 Discover - Stdout lines:', result.stdout);
            console.log('🔍 Discover - Stderr lines:', result.stderr);

            if (result.exitCode === 0) {
                // Look for device array JSON (starts with '[') 
                // Skip structured log lines that start with '{'
                let devices: Device[] = [];
                for (let i = result.stdout.length - 1; i >= 0; i--) {
                    const line = result.stdout[i].trim();
                    console.log(`🔍 Checking line ${i}: "${line.substring(0, 100)}..."`);
                    try {
                        // Only parse lines that start with '[' (device arrays)
                        if (line.startsWith('[')) {
                            console.log('🎯 Found array line, parsing...');
                            const parsed = JSON.parse(line);
                            if (Array.isArray(parsed)) {
                                devices = parsed;
                                console.log('✅ Successfully parsed devices:', devices);
                                break;
                            }
                        }
                    } catch (e) {
                        console.log(`❌ Parse error on line ${i}:`, e);
                        // Continue searching for valid JSON
                        continue;
                    }
                }

                dispatch({ type: 'SET_DEVICES', payload: devices });
                console.log('📊 Final devices sent to context:', devices);
                return devices;
            } else {
                throw new Error(result.stderr.join('\n') || 'Failed to discover devices');
            }
        } finally {
            dispatch({ type: 'SET_OPERATION', payload: null });
        }
    }, [run, dispatch]);

    const planWipe = useCallback(async (options: WipePlanOptions): Promise<WipePlan> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Creating wipe plan...' });

        try {
            const args = ['wipe', '--device', options.device, '--format', 'json'];

            if (options.samples) {
                args.push('--samples', String(options.samples));
            }
            if (options.isoMode) {
                args.push('--iso-mode');
            }
            if (options.noEnrich) {
                args.push('--no-enrich');
            }

            const result = await run(args);
            console.log('🗂️ PlanWipe - Full result:', result);
            console.log('🗂️ PlanWipe - Exit code:', result.exitCode);
            console.log('🗂️ PlanWipe - Stdout lines:', result.stdout);

            if (result.exitCode === 0) {
                // Look for wipe plan JSON object (starts with '{' but not structured logs)
                // Structured logs have "level", "message", "timestamp" fields
                let wipePlan: WipePlan | null = null;
                for (let i = result.stdout.length - 1; i >= 0; i--) {
                    const line = result.stdout[i].trim();
                    console.log(`🗂️ Checking line ${i}: "${line.substring(0, 100)}..."`);
                    try {
                        if (line.startsWith('{')) {
                            const parsed = JSON.parse(line);
                            // Skip structured log messages
                            if (parsed.level && parsed.message && parsed.timestamp) {
                                console.log('⏭️ Skipping structured log message');
                                continue;
                            }
                            // This should be a wipe plan object
                            console.log('🎯 Found wipe plan object:', parsed);
                            wipePlan = parsed;
                            break;
                        }
                    } catch (e) {
                        console.log(`❌ Parse error on line ${i}:`, e);
                        continue;
                    }
                }

                if (!wipePlan) {
                    console.log('❌ No valid wipe plan found in output');
                    throw new Error('No valid wipe plan found in output');
                }

                dispatch({ type: 'SET_WIPE_PLAN', payload: wipePlan });
                console.log('📊 Final wipe plan sent to context:', wipePlan);
                return wipePlan;
            } else {
                throw new Error(result.stderr.join('\n') || 'Failed to create wipe plan');
            }
        } finally {
            dispatch({ type: 'SET_OPERATION', payload: null });
        }
    }, [run, dispatch]);

    const backup = useCallback(async (options: BackupOptions): Promise<BackupResult> => {
        dispatch({ type: 'SET_OPERATION', payload: 'Running backup...' });

        try {
            const args = ['backup', '--device', options.device, '--dest', options.dest];

            if (options.includePaths?.length) {
                args.push('--paths', options.includePaths.join(','));
            }
            if (options.sign) {
                args.push('--sign');
            }
            if (options.signKeyPath) {
                args.push('--sign-key-path', options.signKeyPath);
            }

            const result = await run(args);

            if (result.exitCode === 0) {
                // Parse the backup result from output
                const backupResult = parseBackupResult(result.stdout);
                dispatch({ type: 'SET_BACKUP_RESULT', payload: backupResult });
                return backupResult;
            } else {
                throw new Error(result.stderr.join('\n') || 'Backup failed');
            }
        } finally {
            dispatch({ type: 'SET_OPERATION', payload: null });
        }
    }, [run, dispatch]);

    return {
        logs,
        running,
        run,
        discover,
        planWipe,
        backup,
        clearLogs,
    };
}

function parseBackupResult(stdout: string[]): BackupResult {
    const result: BackupResult = {
        backup_path: '',
        manifest_path: '',
        integrity_checks: 0,
    };

    // Look for certificate paths and other info in stdout
    for (const line of stdout) {
        if (line.includes('Backup path:')) {
            result.backup_path = line.split('Backup path:')[1]?.trim() || '';
        } else if (line.includes('Manifest path:')) {
            result.manifest_path = line.split('Manifest path:')[1]?.trim() || '';
        } else if (line.includes('Certificate JSON:')) {
            result.certificate_json_path = line.split('Certificate JSON:')[1]?.trim() || '';
        } else if (line.includes('Certificate PDF:')) {
            result.certificate_pdf_path = line.split('Certificate PDF:')[1]?.trim() || '';
        } else if (line.includes('Manifest SHA256:')) {
            result.manifest_sha256 = line.split('Manifest SHA256:')[1]?.trim() || '';
        } else if (line.includes('Integrity checks:')) {
            const checksStr = line.split('Integrity checks:')[1]?.trim() || '0';
            result.integrity_checks = parseInt(checksStr, 10) || 0;
        }

        // Try to parse as JSON in case the CLI outputs structured data
        try {
            if (line.trim().startsWith('{')) {
                const parsed = JSON.parse(line);
                if (parsed.backup_path) {
                    Object.assign(result, parsed);
                    break;
                }
            }
        } catch (e) {
            // Continue searching
        }
    }

    return result;
}
