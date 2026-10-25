import QRPreview from '../QRPreview';
import { certTypeIcon, certTypeLabel, formatCertDate } from '../../lib/certificates';
import type { Certificate } from '../../lib/certificates';

interface CertificateCardProps {
    certificate: Certificate;
    certificates: Certificate[];
    generatingPdf: boolean;
    onVerify: (certificate: Certificate) => void;
    onOpenJson: (certificate: Certificate) => void;
    onOpenPdf: (certificate: Certificate) => void;
    onGeneratePdf: (certificate: Certificate) => void;
    onOpenBackupsFolder: () => void;
    onSelect: (certId: string) => void;
}

/** One certificate with its QR code and actions, plus a switcher when there are several. */
function CertificateCard({
    certificate,
    certificates,
    generatingPdf,
    onVerify,
    onOpenJson,
    onOpenPdf,
    onGeneratePdf,
    onOpenBackupsFolder,
    onSelect,
}: CertificateCardProps) {
    return (
        <div className="card" style={{ padding: '2rem' }}>
            <div style={{ textAlign: 'center', marginBottom: '2rem' }}>
                <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>
                    {certTypeIcon(certificate.type)}
                </div>
                <h3 className="font-semibold mb-2">{certTypeLabel(certificate.type)}</h3>
                <p style={{ color: '#64748b', fontSize: '0.9rem' }}>
                    Created: {formatCertDate(certificate.created)}
                </p>
                <p style={{ color: '#64748b', fontSize: '0.75rem', fontFamily: 'monospace' }}>
                    ID: {certificate.id}
                </p>
            </div>

            {/* QR Code in Center */}
            <div style={{ textAlign: 'center', marginBottom: '2rem' }}>
                <QRPreview
                    data={certificate.verifyUrl}
                    title="Verification QR Code"
                    size={200}
                />
                <p style={{ color: '#64748b', fontSize: '0.8rem', marginTop: '1rem' }}>
                    Scan with mobile device to verify certificate authenticity
                </p>
            </div>

            {/* Action Buttons */}
            <div className="space-y-3" style={{ maxWidth: '300px', margin: '0 auto' }}>
                <button
                    className="btn btn-primary"
                    onClick={() => onVerify(certificate)}
                    style={{ width: '100%' }}
                    disabled={generatingPdf}
                >
                    🔍 Verify Online
                </button>

                <button
                    className="btn btn-secondary"
                    onClick={() => onOpenJson(certificate)}
                    style={{ width: '100%' }}
                >
                    📄 Open JSON Certificate
                </button>

                {certificate.pdfPath ? (
                    <button
                        className="btn btn-secondary"
                        onClick={() => onOpenPdf(certificate)}
                        style={{ width: '100%' }}
                    >
                        📋 Open PDF Certificate
                    </button>
                ) : (
                    <button
                        className="btn btn-secondary"
                        onClick={() => onGeneratePdf(certificate)}
                        style={{ width: '100%' }}
                        disabled={generatingPdf}
                    >
                        {generatingPdf ? '⏳ Generating High-Quality PDF...' : '📋 Generate PDF'}
                    </button>
                )}

                <button
                    className="btn btn-outline"
                    onClick={onOpenBackupsFolder}
                    style={{ width: '100%' }}
                    title="Open the folder containing all PDF certificates"
                >
                    📁 Open Backups Folder
                </button>
            </div>

            {/* Certificate Switching (if multiple certificates exist) */}
            {certificates.length > 1 && (
                <div style={{ textAlign: 'center', marginTop: '2rem', paddingTop: '1.5rem', borderTop: '1px solid #e2e8f0' }}>
                    <p style={{ color: '#64748b', fontSize: '0.9rem', marginBottom: '1rem' }}>
                        You have {certificates.length} certificates. Select one to view:
                    </p>
                    <div className="flex gap-2 justify-center flex-wrap">
                        {certificates.map((cert, index) => (
                            <button
                                key={cert.id}
                                className={`btn text-sm ${certificate.id === cert.id ? 'btn-primary' : 'btn-secondary'}`}
                                onClick={() => onSelect(cert.id)}
                            >
                                {certTypeIcon(cert.type)} {certTypeLabel(cert.type)} #{index + 1}
                            </button>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

export default CertificateCard;
