/** Shapes of the events and results that cross the Tauri boundary. */

/** One line the securewipe CLI printed, as the desktop backend emits it. */
export interface LogEvent {
    line: string;
    ts: string;
    stream: 'stdout' | 'stderr';
}

/** Emitted once when the CLI process ends. `code` is null when a signal killed it. */
export interface ExitEvent {
    code: number | null;
    ts: string;
}

/** Everything one CLI run printed, split by stream. */
export interface RunResult {
    exitCode: number;
    stdout: string[];
    stderr: string[];
}

/** What the verification portal's POST /verify returns. */
export interface VerifyResult {
    schema_valid: boolean | null;
    signature_valid: boolean | null;
    hash_valid: boolean | null;
    chain_valid: boolean | null;
    cert_summary: {
        cert_id: string;
        cert_type: string;
        device_model: string;
        policy_method?: string;
        destination?: string;
    };
    computed: Record<string, string | null>;
    errors: string[];
}

export const CLI_EVENTS = {
    stdout: 'securewipe://stdout',
    stderr: 'securewipe://stderr',
    exit: 'securewipe://exit',
} as const;
