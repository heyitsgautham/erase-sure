import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useApp } from '../contexts/AppContext';
import { CLI_EVENTS } from '../types/securewipe';
import type { LogEvent } from '../types/securewipe';

/**
 * stderr lines the CLI prints when no signing key is configured.
 * Unsigned runs are normal in development, so these stay out of the visible log.
 */
const HIDDEN_STDERR = ['Failed to load signing key', 'SECUREWIPE_SIGN_KEY_PATH not set'];

export function isHiddenLine(event: LogEvent): boolean {
    return event.stream === 'stderr' && HIDDEN_STDERR.some((text) => event.line.includes(text));
}

/**
 * Copy every CLI output line into the app log, once.
 *
 * Mount this a single time near the root. Screens read `state.logs` and never
 * subscribe to the CLI streams themselves, so a line cannot be logged twice.
 */
export function useCliLogStream() {
    const { addLog } = useApp();

    useEffect(() => {
        let disposed = false;
        const unlisteners: UnlistenFn[] = [];

        const subscribe = async (event: string, stream: LogEvent['stream']) => {
            const unlisten = await listen<LogEvent>(event, ({ payload }) => {
                const logEvent: LogEvent = { ...payload, stream };
                if (!isHiddenLine(logEvent)) addLog(logEvent);
            });
            // The component may have unmounted while listen() was in flight.
            if (disposed) unlisten();
            else unlisteners.push(unlisten);
        };

        void subscribe(CLI_EVENTS.stdout, 'stdout');
        void subscribe(CLI_EVENTS.stderr, 'stderr');

        return () => {
            disposed = true;
            unlisteners.forEach((unlisten) => unlisten());
        };
    }, [addLog]);
}
