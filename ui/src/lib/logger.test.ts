import { describe, expect, it, vi } from 'vitest';
import { createLogger, describeError } from './logger';
import type { LogLevel } from './logger';

const fixedNow = () => new Date('2026-01-01T00:00:00Z');

function capture(debug = true) {
    const lines: [LogLevel, unknown][] = [];
    const log = createLogger('backup', {
        debug,
        now: fixedNow,
        sink: (level, line) => lines.push([level, JSON.parse(line)]),
    });
    return { log, lines };
}

describe('createLogger', () => {
    it('writes one JSON entry with timestamp, level, module and message', () => {
        const { log, lines } = capture();

        log.info('backup started', { device: '/dev/sdb' });

        expect(lines).toEqual([
            [
                'info',
                {
                    ts: '2026-01-01T00:00:00.000Z',
                    level: 'info',
                    module: 'backup',
                    msg: 'backup started',
                    data: { device: '/dev/sdb' },
                },
            ],
        ]);
    });

    it('attaches the error that caused a warning or failure', () => {
        const { log, lines } = capture();

        log.warn('retrying', undefined, 'busy');
        log.error('backup failed', { attempt: 2 }, new TypeError('bad path'));

        expect(lines[0][1]).toMatchObject({ level: 'warn', error: { name: 'Error', message: 'busy' } });
        expect(lines[1][1]).toMatchObject({
            level: 'error',
            data: { attempt: 2 },
            error: { name: 'TypeError', message: 'bad path' },
        });
    });

    it('leaves out empty data', () => {
        const { log, lines } = capture();

        log.info('ready', {});

        expect(lines[0][1]).not.toHaveProperty('data');
    });

    it('drops debug entries when debug is off', () => {
        const { log, lines } = capture(false);

        log.debug('noisy detail');
        log.info('kept');

        expect(lines.map(([level]) => level)).toEqual(['info']);
    });

    it('writes to the console method for the level by default', () => {
        const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});

        createLogger('ui').warn('careful');

        expect(warn).toHaveBeenCalledTimes(1);
        expect(JSON.parse(warn.mock.calls[0][0] as string)).toMatchObject({
            level: 'warn',
            module: 'ui',
            msg: 'careful',
        });
    });
});

describe('describeError', () => {
    it('handles errors, strings, plain objects and values JSON cannot encode', () => {
        expect(describeError(new RangeError('too big'))).toEqual({ name: 'RangeError', message: 'too big' });
        expect(describeError('Session not found')).toEqual({ name: 'Error', message: 'Session not found' });
        expect(describeError({ code: 5 })).toEqual({ name: 'Error', message: '{"code":5}' });

        const circular: Record<string, unknown> = {};
        circular.self = circular;
        expect(describeError(circular)).toEqual({ name: 'Error', message: '[object Object]' });
    });
});
