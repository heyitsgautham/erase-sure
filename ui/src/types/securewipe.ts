export interface LogEvent {
    line: string;
    ts: string;
    stream: 'stdout' | 'stderr';
}

export interface ExitEvent {
    code: number | null;
    ts: string;
}

export interface RunResult {
    exitCode: number | null;
    stdout: string[];
    stderr: string[];
}

export interface Device {
    path: string;
    model: string;
    serial: string;
    capacity: number;
    bus: string;
    mountpoints: string[];
    risk_level: 'SAFE' | 'HIGH' | 'CRITICAL';
    blocked: boolean;
    block_reason?: string;
}

export interface WipePlan {
    device_path: string;
    policy: 'PURGE' | 'CLEAR' | 'DESTROY';
    main_method: string;
    hpa_dco_clear: boolean;
    verification: {
        samples: number;
    };
    blocked: boolean;
    block_reason?: string;
}

export interface BackupOptions {
    device: string;
    dest: string;
    sign?: boolean;
    signKeyPath?: string;
    includePaths?: string[];
}

export interface BackupResult {
    backup_path: string;
    manifest_path: string;
    integrity_checks: number;
    certificate_json_path?: string;
    certificate_pdf_path?: string;
    manifest_sha256?: string;
}

export interface WipePlanOptions {
    device: string;
    samples?: number;
    isoMode?: boolean;
    noEnrich?: boolean;
}

export interface DiscoverOptions {
    format?: 'json' | 'table';
    noEnrich?: boolean;
}