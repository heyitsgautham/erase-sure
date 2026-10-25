/**
 * Structured client logging.
 *
 * Every entry is one JSON object with a timestamp, level, module and message,
 * matching the JSON lines the Rust CLI writes through `tracing`. Debug entries
 * are dropped from production builds.
 */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
    ts: string;
    level: LogLevel;
    module: string;
    msg: string;
    data?: Record<string, unknown>;
    error?: { name: string; message: string };
}

export interface Logger {
    debug(msg: string, data?: Record<string, unknown>): void;
    info(msg: string, data?: Record<string, unknown>): void;
    warn(msg: string, data?: Record<string, unknown>, error?: unknown): void;
    error(msg: string, data?: Record<string, unknown>, error?: unknown): void;
}

/** Reduce anything thrown to a name and message that survive JSON.stringify. */
export function describeError(error: unknown): { name: string; message: string } {
    if (error instanceof Error) return { name: error.name, message: error.message };
    if (typeof error === 'string') return { name: 'Error', message: error };
    try {
        return { name: 'Error', message: JSON.stringify(error) };
    } catch {
        return { name: 'Error', message: String(error) };
    }
}

export interface LoggerOptions {
    /** Write debug entries. Defaults to true outside production builds. */
    debug?: boolean;
    /** Where entries go. Defaults to the console method for the level. */
    sink?: (level: LogLevel, line: string) => void;
    now?: () => Date;
}

const consoleSink = (level: LogLevel, line: string) => {
    // eslint-disable-next-line no-console
    console[level](line);
};

export function createLogger(module: string, options: LoggerOptions = {}): Logger {
    const debugEnabled = options.debug ?? !import.meta.env.PROD;
    const sink = options.sink ?? consoleSink;
    const now = options.now ?? (() => new Date());

    const write = (level: LogLevel, msg: string, data?: Record<string, unknown>, error?: unknown) => {
        if (level === 'debug' && !debugEnabled) return;
        const entry: LogEntry = { ts: now().toISOString(), level, module, msg };
        if (data && Object.keys(data).length > 0) entry.data = data;
        if (error !== undefined) entry.error = describeError(error);
        sink(level, JSON.stringify(entry));
    };

    return {
        debug: (msg, data) => write('debug', msg, data),
        info: (msg, data) => write('info', msg, data),
        warn: (msg, data, error) => write('warn', msg, data, error),
        error: (msg, data, error) => write('error', msg, data, error),
    };
}
