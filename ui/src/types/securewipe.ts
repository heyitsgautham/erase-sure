export interface LogEvent {
    line: string;
    ts: string;
    stream: 'stdout' | 'stderr';
}

export interface ExitEvent {
    code: number | null;
    ts: string;
    session_id: string;
}

export interface RunResult {
    exitCode: number | null;
    stdout: string[];
    stderr: string[];
    sessionId: string;
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
    policy: 'CLEAR' | 'PURGE' | 'DESTROY';
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

export interface WipePlanOptions {
    device: string;
    samples?: number;
    isoMode?: boolean;
    noEnrich?: boolean;
}

export interface BackupResult {
    certPathJson?: string;
    certPathPdf?: string;
    manifestSha256?: string;
    backupPath?: string;
    filesProcessed?: number;
    totalSize?: number;
}

export interface CertificateInfo {
    cert_id: string;
    cert_type: 'backup' | 'wipe';
    created_at: string;
    device?: {
        path: string;
        model: string;
        serial: string;
    };
}
