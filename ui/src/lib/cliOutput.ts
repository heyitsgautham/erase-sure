/**
 * Parsing for what the securewipe CLI prints.
 *
 * The CLI writes its result as JSON on stdout. Older builds also wrote JSON log
 * lines there, so the parser skips any line that looks like a log entry.
 */
import type { Device, WipePlan } from '../contexts/AppContext';

/** One device as `securewipe discover --format json` prints it. */
export interface CliDevice {
    name?: string;
    path?: string;
    model?: string | null;
    serial?: string | null;
    capacity_bytes?: number;
    capacity?: number;
    bus?: string | null;
    mountpoints?: string[];
    risk_level?: Device['risk_level'];
}

/** The plan `securewipe wipe` prints, mirroring WipePlan in core/src/wipe/plan.rs. */
export interface CliWipePlan {
    device?: string;
    policy?: WipePlan['policy'];
    main_method?: string;
    hpa_dco_clear?: boolean;
    verification?: { samples?: number };
    blocked?: boolean;
    reason?: string | null;
}

/** Parse the JSON result out of CLI stdout lines, skipping JSON log lines. */
export function parseJsonOutput<T>(lines: string[]): T {
    const nonLogLines: string[] = [];

    for (const line of lines) {
        if (!line.trim()) continue;

        try {
            const parsed = JSON.parse(line);
            // A log entry has both a level and a timestamp.
            if (parsed.level && parsed.timestamp) {
                continue;
            }
            nonLogLines.push(line);
        } catch {
            // Not JSON on its own, so probably one line of pretty-printed JSON.
            nonLogLines.push(line);
        }
    }

    if (nonLogLines.length === 0) {
        throw new Error('No JSON output found in command output');
    }

    const jsonContent = nonLogLines.join('');
    try {
        return JSON.parse(jsonContent);
    } catch (error) {
        // Fall back to the last line that parses on its own.
        for (let i = nonLogLines.length - 1; i >= 0; i--) {
            try {
                return JSON.parse(nonLogLines[i]);
            } catch {
                continue;
            }
        }
        throw new Error(`Failed to parse JSON output: ${error}`);
    }
}

/** Map a discovered device to the UI model; CRITICAL devices are blocked. */
export function toDevice(device: CliDevice): Device {
    return {
        path: device.name || device.path || '',
        model: device.model || 'Unknown Device',
        serial: device.serial || 'N/A',
        capacity: device.capacity_bytes || device.capacity || 0,
        bus: device.bus || 'Unknown',
        mountpoints: device.mountpoints || [],
        risk_level: device.risk_level || 'SAFE',
        blocked: device.risk_level === 'CRITICAL',
        block_reason: device.risk_level === 'CRITICAL' ? 'System disk with active mount points' : undefined,
    };
}

/** Map the CLI's wipe plan to the UI model, filling gaps from what the user asked for. */
export function toWipePlan(plan: CliWipePlan, requestedDevice: string, requestedSamples?: number): WipePlan {
    return {
        device_path: plan.device || requestedDevice,
        policy: plan.policy || 'PURGE',
        main_method: plan.main_method || 'Unknown Method',
        hpa_dco_clear: plan.hpa_dco_clear || false,
        verification: {
            samples: plan.verification?.samples || requestedSamples || 128,
        },
        blocked: plan.blocked || false,
        block_reason: plan.reason ?? undefined,
    };
}

export interface WipePlanOptions {
    device: string;
    samples?: number;
    isoMode?: boolean;
    noEnrich?: boolean;
}

/** Arguments for a wipe plan. `--format json` keeps the CLI in planning mode; it never wipes. */
export function buildWipePlanArgs(opts: WipePlanOptions): string[] {
    const args = ['wipe', '--device', opts.device, '--format', 'json'];
    if (opts.samples) args.push('--samples', String(opts.samples));
    if (opts.isoMode) args.push('--iso-mode');
    if (opts.noEnrich) args.push('--no-enrich');
    return args;
}
