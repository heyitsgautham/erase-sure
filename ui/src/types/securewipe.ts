import type { z } from 'zod';
import type { VerifyResultSchema } from '../lib/schemas';

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
export type VerifyResult = z.infer<typeof VerifyResultSchema>;

export const CLI_EVENTS = {
    stdout: 'securewipe://stdout',
    stderr: 'securewipe://stderr',
    exit: 'securewipe://exit',
} as const;
