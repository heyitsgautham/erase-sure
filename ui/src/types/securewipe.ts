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
    name: string;  // Actual field from CLI output: "/dev/zram0"
    model: string | null;  // Actual field from CLI output
    serial: string | null;  // Actual field from CLI output
    capacity_bytes: number;  // Actual field from CLI output
    bus: string | null;  // Actual field from CLI output
    mountpoints: string[];  // Actual field from CLI output
    risk_level: 'SAFE' | 'HIGH' | 'CRITICAL';  // Actual field from CLI output
}

export interface WipePlan {
    device_name: string;  // Updated to match CLI output
    policy: 'PURGE' | 'CLEAR' | 'DESTROY';
    main_method: string;
    hpa_dco_clear?: boolean;
    verification?: {
        samples: number;
    };
    steps?: any[];  // Added steps array
    blocked?: boolean;
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