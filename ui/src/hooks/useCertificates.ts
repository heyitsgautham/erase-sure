import { useCallback, useEffect, useState } from 'react';
import { commands } from '../lib/commands';
import { CertificateSummarySchema, describeIssues } from '../lib/schemas';
import { certificateDir, newestFirst, pdfCandidates, toCertificate } from '../lib/certificates';
import type { Certificate } from '../lib/certificates';
import { describeError, createLogger } from '../lib/logger';
import { PORTAL_URL } from '../lib/config';

const log = createLogger('certificates');

async function firstExisting(paths: string[]): Promise<string | undefined> {
    for (const path of paths) {
        if (await commands.fileExists(path)) return path;
    }
    return undefined;
}

/** Read one certificate file. Returns null, with a warning, for a file that is not a certificate. */
async function loadCertificate(path: string, homeDir: string): Promise<Certificate | null> {
    try {
        const parsed = CertificateSummarySchema.safeParse(JSON.parse(await commands.readFileContent(path)));
        if (!parsed.success) {
            log.warn('skipping file that is not a certificate', { file: path, problem: describeIssues(parsed.error) });
            return null;
        }
        const pdfPath = await firstExisting(pdfCandidates(path, parsed.data.cert_id, homeDir));
        return toCertificate(path, parsed.data, pdfPath, PORTAL_URL);
    } catch (error) {
        log.warn('skipping unreadable certificate', { file: path }, error);
        return null;
    }
}

/**
 * The certificates in `~/SecureWipe/certificates`, newest first.
 *
 * A file that is not a valid certificate is skipped. When the folder itself
 * cannot be read, `error` says why and the list stays empty.
 */
export function useCertificates() {
    const [certificates, setCertificates] = useState<Certificate[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const reload = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const homeDir = await commands.getHomeDir();
            const files = await commands.listCertFiles(certificateDir(homeDir));
            const loaded = await Promise.all(files.map((file) => loadCertificate(file, homeDir)));
            setCertificates(newestFirst(loaded.filter((cert): cert is Certificate => cert !== null)));
        } catch (err) {
            log.error('loading certificates failed', undefined, err);
            setCertificates([]);
            setError(`Could not read the certificates folder: ${describeError(err).message}`);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        void reload();
    }, [reload]);

    /** Record a freshly generated PDF without reloading the whole list. */
    const setPdfPath = useCallback((certId: string, pdfPath: string) => {
        setCertificates((prev) => prev.map((cert) => (cert.id === certId ? { ...cert, pdfPath } : cert)));
    }, []);

    return { certificates, loading, error, reload, setPdfPath };
}
