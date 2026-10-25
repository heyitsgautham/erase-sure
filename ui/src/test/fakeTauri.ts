/**
 * A fake Tauri backend for component and hook tests.
 *
 * It answers `invoke()` calls from per-command handlers and delivers events to
 * whatever `listen()` registered, so tests drive the UI exactly as the desktop
 * backend would. Nothing here spawns the securewipe CLI or touches a device.
 */
import { mockIPC } from '@tauri-apps/api/mocks';

export type CommandHandler = (args: Record<string, unknown>) => unknown;

export interface FakeCall {
    cmd: string;
    args: Record<string, unknown>;
}

/** What the fake CLI prints for one `run_securewipe` call. */
export interface FakeCliRun {
    stdout?: string[];
    stderr?: string[];
    code?: number | null;
}

export interface FakeTauri {
    /** Every command the UI invoked, in order. Event plumbing is left out. */
    calls: FakeCall[];
    /** Answer `cmd` with `handler`. A handler that throws becomes a rejected invoke. */
    on(cmd: string, handler: CommandHandler): void;
    /** Deliver an event to every listener registered for it. */
    emit(event: string, payload: unknown): void;
    /** How many listeners are registered for `event` right now. */
    listenerCount(event: string): number;
    /**
     * Answer `run_securewipe` like the desktop backend: stream the lines, then emit the exit event.
     * The lines arrive before the invoke resolves, which is what a fast CLI does in the real app.
     */
    cli(script: (args: string[]) => FakeCliRun): void;
    /** Calls made with this command name. */
    callsTo(cmd: string): FakeCall[];
}

const TS = '2026-01-01T00:00:00Z';

export function installFakeTauri(): FakeTauri {
    const handlers = new Map<string, CommandHandler>();
    const listeners = new Map<string, number[]>();
    const calls: FakeCall[] = [];
    let nextEventId = 1;

    const emit = (event: string, payload: unknown) => {
        for (const handlerId of [...(listeners.get(event) ?? [])]) {
            const callback = (window as unknown as Record<string, unknown>)[`_${handlerId}`];
            if (typeof callback === 'function') {
                callback({ event, windowLabel: null, payload, id: nextEventId++ });
            }
        }
    };

    mockIPC((cmd, args) => {
        if (cmd === 'tauri') {
            return handleModule(args as Record<string, unknown>);
        }
        calls.push({ cmd, args });
        const handler = handlers.get(cmd);
        if (!handler) {
            throw new Error(`fake Tauri has no handler for '${cmd}'`);
        }
        return handler(args);
    });

    function handleModule(args: Record<string, unknown>): unknown {
        const message = (args.message ?? {}) as Record<string, unknown>;
        if (args.__tauriModule === 'Event' && message.cmd === 'listen') {
            const event = String(message.event);
            const handlerId = Number(message.handler);
            listeners.set(event, [...(listeners.get(event) ?? []), handlerId]);
            return handlerId;
        }
        if (args.__tauriModule === 'Event' && message.cmd === 'unlisten') {
            const event = String(message.event);
            listeners.set(
                event,
                (listeners.get(event) ?? []).filter((id) => id !== Number(message.eventId)),
            );
            return null;
        }
        const key = `${String(args.__tauriModule).toLowerCase()}.${String(message.cmd)}`;
        calls.push({ cmd: key, args: message });
        const handler = handlers.get(key);
        if (!handler) {
            throw new Error(`fake Tauri has no handler for '${key}'`);
        }
        return handler(message);
    }

    return {
        calls,
        on: (cmd, handler) => {
            handlers.set(cmd, handler);
        },
        emit,
        listenerCount: (event) => listeners.get(event)?.length ?? 0,
        cli: (script) => {
            handlers.set('run_securewipe', (args) => {
                const run = script(args.args as string[]);
                for (const line of run.stdout ?? []) emit('securewipe://stdout', { line, ts: TS, stream: 'stdout' });
                for (const line of run.stderr ?? []) emit('securewipe://stderr', { line, ts: TS, stream: 'stderr' });
                emit('securewipe://exit', { code: run.code === undefined ? 0 : run.code, ts: TS });
                return null;
            });
        },
        callsTo: (cmd) => calls.filter((call) => call.cmd === cmd),
    };
}
