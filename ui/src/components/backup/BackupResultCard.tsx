import type { BackupResult } from '../../contexts/AppContext';
import FileLink from '../FileLink';
import { PORTAL_URL } from '../../lib/config';

interface BackupResultCardProps {
    result: BackupResult;
    onViewCertificates: () => void;
}

/** Where the finished backup went and links to the files it produced. */
function BackupResultCard({ result, onViewCertificates }: BackupResultCardProps) {
    return (
        <div className="mb-8">
            <div className="card" style={{ borderRadius: '16px', padding: '2rem' }}>
                <div className="flex items-center gap-4 mb-6">
                    <div
                        style={{
                            width: '48px',
                            height: '48px',
                            background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                            borderRadius: '12px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: '24px',
                        }}
                    >
                        ✅
                    </div>
                    <div>
                        <h2 className="font-semibold" style={{ fontSize: '1.5rem', color: '#059669' }}>
                            Backup Completed Successfully
                        </h2>
                        <p style={{ color: '#64748b' }}>Your data has been securely encrypted and backed up</p>
                    </div>
                </div>

                <div className="grid grid-cols-2 gap-6 mb-6">
                    <div>
                        <h3 className="font-semibold mb-3" style={{ fontSize: '1.125rem' }}>
                            Backup Summary
                        </h3>
                        <div className="space-y-3">
                            <div>
                                <span className="font-medium">Location:</span>
                                <div className="mt-1 p-2 bg-gray-50 rounded font-mono text-sm">
                                    {result.backup_path}
                                </div>
                            </div>
                            <div>
                                <span className="font-medium">Integrity Checks:</span>
                                <div className="mt-1 flex items-center gap-2">
                                    <span style={{ color: '#10b981', fontSize: '1.125rem' }}>✓</span>
                                    <span>{result.integrity_checks} samples verified</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h3 className="font-semibold mb-3" style={{ fontSize: '1.125rem' }}>
                            Generated Files
                        </h3>
                        <div className="space-y-3">
                            <FileLink path={result.backup_path} label="Open Backup Folder" type="folder" />
                            <FileLink path={result.manifest_path} label="Backup Manifest" type="json" />
                        </div>
                    </div>
                </div>

                <div className="grid grid-cols-2 gap-4 mb-6">
                    {result.certificate_json_path && (
                        <FileLink path={result.certificate_json_path} label="Backup Certificate (JSON)" type="json" />
                    )}
                    {result.certificate_pdf_path && (
                        <FileLink path={result.certificate_pdf_path} label="Backup Certificate (PDF)" type="pdf" />
                    )}
                </div>

                <div className="flex gap-4">
                    <button
                        className="btn btn-primary"
                        onClick={onViewCertificates}
                        style={{ borderRadius: '8px', fontWeight: '600' }}
                    >
                        📜 View All Certificates
                    </button>

                    <button
                        className="btn btn-secondary"
                        onClick={() => window.open(PORTAL_URL, '_blank')}
                        style={{ borderRadius: '8px', fontWeight: '600' }}
                    >
                        🌐 Open in Portal Verify
                    </button>
                </div>
            </div>
        </div>
    );
}

export default BackupResultCard;
