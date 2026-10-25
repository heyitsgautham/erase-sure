/**
 * Where certificates live on disk and how the list describes them.
 *
 * The CLI writes `~/SecureWipe/certificates/<cert_id>.json`. A PDF export goes
 * next to it, and the desktop app copies it to `~/SecureWipe/backups/`.
 */
import type { z } from 'zod';
import type { CertificateSummarySchema } from './schemas';

export type CertificateSummary = z.infer<typeof CertificateSummarySchema>;

export interface Certificate {
    id: string;
    type: 'backup' | 'wipe';
    filename: string;
    path: string;
    pdfPath?: string;
    created: Date;
    verifyUrl: string;
}

export function certificateDir(homeDir: string): string {
    return `${homeDir}/SecureWipe/certificates`;
}

export function backupsDir(homeDir: string): string {
    return `${homeDir}/SecureWipe/backups`;
}

/** Places a certificate's PDF may be, most preferred first. */
export function pdfCandidates(jsonPath: string, certId: string, homeDir: string): string[] {
    return [`${backupsDir(homeDir)}/${certId}.pdf`, jsonPath.replace(/\.json$/, '.pdf')];
}

export function verifyUrl(portalUrl: string, certId: string): string {
    return `${portalUrl}/verify?cert_id=${encodeURIComponent(certId)}`;
}

export function toCertificate(
    path: string,
    summary: CertificateSummary,
    pdfPath: string | undefined,
    portalUrl: string,
): Certificate {
    return {
        id: summary.cert_id,
        type: summary.cert_type,
        filename: path.split('/').pop() || path,
        path,
        pdfPath,
        created: new Date(summary.created_at),
        verifyUrl: verifyUrl(portalUrl, summary.cert_id),
    };
}

export function newestFirst(certificates: Certificate[]): Certificate[] {
    return [...certificates].sort((a, b) => b.created.getTime() - a.created.getTime());
}

export function certTypeLabel(type: Certificate['type']): string {
    return type === 'backup' ? 'Backup Certificate' : 'Wipe Certificate';
}

export function certTypeIcon(type: Certificate['type']): string {
    return type === 'backup' ? '📦' : '🗑️';
}

export function formatCertDate(date: Date): string {
    return date.toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}
