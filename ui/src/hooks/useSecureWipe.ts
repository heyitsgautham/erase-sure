import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
    LogEvent,
    ExitEvent,
    RunResult,
    Device,
    WipePlan,
    BackupOptions,
    WipePlanOptions,
    BackupResult,
} from '../types/securewipe';

const MAX_LOG_LINES = 2000;

export function useSecureWipe() {
    const [logs, setLogs] = useState<LogEvent[]>([]);
    const [running, setRunning] = useState(false);
    const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);

    const unlistenFuncsRef = useRef<UnlistenFn[]>([]);
    const pendingPromiseRef = useRef<{
        resolve: (result: RunResult) => void;
        reject: (error: Error) => void;
        stdout: string[];
        stderr: string[];
    } | null>(null);

    // Cleanup function for event listeners
    const cleanupListeners = useCallback(() => {
        unlistenFuncsRef.current.forEach(unlisten => unlisten());
        unlistenFuncsRef.current = [];
    }, []);

    // Setup event listeners when a command starts
    const setupEventListeners = useCallback(async () => {
        cleanupListeners();

        const unlistenStdout = await listen<LogEvent>('securewipe://stdout', (event) => {
            const logEvent = event.payload;

            setLogs(prevLogs => {
                const newLogs = [...prevLogs, logEvent];
                // Keep only the last MAX_LOG_LINES entries
                return newLogs.slice(-MAX_LOG_LINES);
            });

            // Collect stdout for final result
            if (pendingPromiseRef.current) {
                pendingPromiseRef.current.stdout.push(logEvent.line);
            }
        });

        const unlistenStderr = await listen<LogEvent>('securewipe://stderr', (event) => {
            const logEvent = event.payload;

            setLogs(prevLogs => {
                const newLogs = [...prevLogs, logEvent];
                return newLogs.slice(-MAX_LOG_LINES);
            });

            // Collect stderr for final result
            if (pendingPromiseRef.current) {
                pendingPromiseRef.current.stderr.push(logEvent.line);
            }
        });

        const unlistenExit = await listen<ExitEvent>('securewipe://exit', (event) => {
            const exitEvent = event.payload;

            setRunning(false);
            setCurrentSessionId(null);

            // Resolve pending promise with results
            if (pendingPromiseRef.current) {
                const result: RunResult = {
                    exitCode: exitEvent.code,
                    stdout: pendingPromiseRef.current.stdout,
                    stderr: pendingPromiseRef.current.stderr,
                    sessionId: exitEvent.session_id,
                };

                if (exitEvent.code === 0) {
                    pendingPromiseRef.current.resolve(result);
                } else {
                    const errorMessage = pendingPromiseRef.current.stderr.join('\n') ||
                        `Process exited with code ${exitEvent.code}`;
                    pendingPromiseRef.current.reject(new Error(errorMessage));
                }

                pendingPromiseRef.current = null;
            }

            cleanupListeners();
        });

        unlistenFuncsRef.current = [unlistenStdout, unlistenStderr, unlistenExit];
    }, [cleanupListeners]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            cleanupListeners();
            if (currentSessionId) {
                invoke('cancel_securewipe', { sessionId: currentSessionId }).catch(console.error);
            }
        };
    }, [cleanupListeners, currentSessionId]);

    const run = useCallback(async (args: string[]): Promise<RunResult> => {
        if (running) {
            throw new Error('Another command is already running');
        }

        setRunning(true);
        setLogs([]);

        try {
            await setupEventListeners();

            return new Promise((resolve, reject) => {
                pendingPromiseRef.current = {
                    resolve,
                    reject,
                    stdout: [],
                    stderr: [],
                };

                invoke<string>('run_securewipe', { args })
                    .then((sessionId) => {
                        setCurrentSessionId(sessionId);
                    })
                    .catch((error) => {
                        setRunning(false);
                        cleanupListeners();
                        pendingPromiseRef.current = null;
                        reject(new Error(`Failed to start command: ${error}`));
                    });
            });
        } catch (error) {
            setRunning(false);
            cleanupListeners();
            throw error;
        }
    }, [running, setupEventListeners, cleanupListeners]);

    const discover = useCallback(async (): Promise<Device[]> => {
        const result = await run(['discover', '--format', 'json']);

        // Join all stdout lines to handle multi-line JSON
        const fullOutput = result.stdout.join('\n');
        
        // Find the JSON array by looking for content between [ and ]
        const jsonStart = fullOutput.indexOf('[');
        const jsonEnd = fullOutput.lastIndexOf(']');
        
        if (jsonStart === -1 || jsonEnd === -1 || jsonStart >= jsonEnd) {
            console.error('No JSON array found in output:', fullOutput);
            throw new Error('No valid device data found in output');
        }
        
        const jsonString = fullOutput.substring(jsonStart, jsonEnd + 1);
        
        try {
            const rawDevices = JSON.parse(jsonString);
            if (!Array.isArray(rawDevices)) {
                throw new Error('Expected device array');
            }
            
            // Map CLI output format to UI expected format
            const devices: Device[] = rawDevices.map((device: any) => ({
                path: device.name,                    // CLI uses 'name', UI expects 'path'
                model: device.model || '',
                serial: device.serial || '',
                capacity: device.capacity_bytes,      // CLI uses 'capacity_bytes', UI expects 'capacity'
                bus: device.bus || '',
                mountpoints: device.mountpoints || [],
                risk_level: device.risk_level,
                blocked: device.risk_level === 'CRITICAL', // Block critical devices by default
                block_reason: device.risk_level === 'CRITICAL' ? 'Critical system disk' : undefined
            }));
            
            return devices;
        } catch (e) {
            console.error('Failed to parse device JSON:', e);
            console.error('Raw JSON string:', jsonString);
            throw new Error('Failed to parse device data');
        }
    }, [run]);

    const planWipe = useCallback(async (options: WipePlanOptions): Promise<WipePlan> => {
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

        // Parse the last valid JSON object from stdout
        for (let i = result.stdout.length - 1; i >= 0; i--) {
            const line = result.stdout[i].trim();
            if (line.startsWith('{')) {
                try {
                    return JSON.parse(line);
                } catch (e) {
                    continue;
                }
            }
        }

        throw new Error('No valid wipe plan found in output');
    }, [run]);

    const backup = useCallback(async (options: BackupOptions): Promise<BackupResult> => {
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

        // Parse output for certificate paths and other info
        const backupResult: BackupResult = {};

        for (const line of result.stdout) {
            if (line.includes('Certificate JSON:')) {
                backupResult.certPathJson = line.split('Certificate JSON:')[1]?.trim();
            } else if (line.includes('Certificate PDF:')) {
                backupResult.certPathPdf = line.split('Certificate PDF:')[1]?.trim();
            } else if (line.includes('Manifest SHA256:')) {
                backupResult.manifestSha256 = line.split('Manifest SHA256:')[1]?.trim();
            } else if (line.includes('Backup Path:')) {
                backupResult.backupPath = line.split('Backup Path:')[1]?.trim();
            }
        }

        // Try to parse JSON summary if available
        for (let i = result.stdout.length - 1; i >= 0; i--) {
            const line = result.stdout[i].trim();
            if (line.startsWith('{')) {
                try {
                    const summary = JSON.parse(line);
                    if (summary.cert_path_json) backupResult.certPathJson = summary.cert_path_json;
                    if (summary.cert_path_pdf) backupResult.certPathPdf = summary.cert_path_pdf;
                    if (summary.manifest_sha256) backupResult.manifestSha256 = summary.manifest_sha256;
                    if (summary.backup_path) backupResult.backupPath = summary.backup_path;
                    if (summary.files_processed) backupResult.filesProcessed = summary.files_processed;
                    if (summary.total_size) backupResult.totalSize = summary.total_size;
                    break;
                } catch (e) {
                    continue;
                }
            }
        }

        return backupResult;
    }, [run]);

    const cancel = useCallback(async (): Promise<void> => {
        if (currentSessionId) {
            await invoke('cancel_securewipe', { sessionId: currentSessionId });
            setRunning(false);
            setCurrentSessionId(null);
            cleanupListeners();
        }
    }, [currentSessionId, cleanupListeners]);

    const clearLogs = useCallback(() => {
        setLogs([]);
    }, []);

    return {
        logs,
        running,
        run,
        discover,
        planWipe,
        backup,
        cancel,
        clearLogs,
        // Backward compatibility aliases
        discoverDevices: discover,
        createWipePlan: (devicePath: string) => planWipe({ device: devicePath }),
        runBackup: async (devicePath: string, destination: string, signKeyPath?: string) => {
            const options: BackupOptions = {
                device: devicePath,
                dest: destination,
                sign: !!signKeyPath,
            };
            if (signKeyPath) {
                options.signKeyPath = signKeyPath;
            }
            return backup(options);
        },
    };
}
