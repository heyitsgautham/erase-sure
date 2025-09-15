import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import FileLink from '../components/FileLink';
import QRPreview from '../components/QRPreview';

interface Certificate {
    id: string;
    type: 'backup' | 'wipe';
    filename: string;
    path: string;
    pdfPath?: string;
    created: Date;
    verifyUrl?: string;
}

function Certificates() {
    const navigate = useNavigate();
    const [certificates, setCertificates] = useState<Certificate[]>([]);
    const [selectedCert, setSelectedCert] = useState<Certificate | null>(null);

    // Mock certificate loading - in production, scan ~/SecureWipe/certificates/
    useEffect(() => {
        const mockCerts: Certificate[] = [
            {
                id: 'backup_20250911_143022',
                type: 'backup',
                filename: 'backup_cert_20250911_143022.json',
                path: '~/SecureWipe/certificates/backup_cert_20250911_143022.json',
                pdfPath: '~/SecureWipe/certificates/backup_cert_20250911_143022.pdf',
                created: new Date('2024-09-11T14:30:22Z'),
                verifyUrl: 'http://localhost:8000/verify?id=backup_20250911_143022'
            },
            {
                id: 'wipe_20250911_145530',
                type: 'wipe',
                filename: 'wipe_cert_20250911_145530.json',
                path: '~/SecureWipe/certificates/wipe_cert_20250911_145530.json',
                pdfPath: '~/SecureWipe/certificates/wipe_cert_20250911_145530.pdf',
                created: new Date('2024-09-11T14:55:30Z'),
                verifyUrl: 'http://localhost:8000/verify?id=wipe_20250911_145530'
            }
        ];
        setCertificates(mockCerts);
        if (mockCerts.length > 0) {
            setSelectedCert(mockCerts[0]);
        }
    }, []);

    const handleOpenPortalVerify = (cert: Certificate) => {
        if (cert.verifyUrl) {
            window.open(cert.verifyUrl, '_blank');
        } else {
            window.open('http://localhost:8000', '_blank');
        }
    };

    const handleBackToHome = () => {
        navigate('/');
    };

    const handleNewBackup = () => {
        navigate('/discover');
    };

    const formatDate = (date: Date) => {
        return date.toLocaleString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    };

    const getCertTypeIcon = (type: Certificate['type']) => {
        return type === 'backup' ? '📦' : '🗑️';
    };

    const getCertTypeLabel = (type: Certificate['type']) => {
        return type === 'backup' ? 'Backup Certificate' : 'Wipe Certificate';
    };

    return (
        <div style={{ maxWidth: '1400px', margin: '0 auto' }}>
            <div className="mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    Certificate Management
                </h2>
                <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                    View, verify, and manage backup and wipe certificates. Each certificate provides
                    cryptographic proof of completed operations and can be verified independently.
                </p>

                <div className="flex gap-4 mb-6">
                    <button
                        className="btn btn-primary"
                        onClick={handleNewBackup}
                    >
                        ➕ Create New Backup
                    </button>

                    <button
                        className="btn btn-secondary"
                        onClick={handleBackToHome}
                    >
                        🏠 Back to Home
                    </button>
                </div>
            </div>

            {certificates.length === 0 ? (
                <div className="card text-center" style={{ padding: '3rem' }}>
                    <div style={{ fontSize: '4rem', marginBottom: '1rem', opacity: 0.3 }}>📜</div>
                    <h3 className="font-semibold mb-2">No Certificates Found</h3>
                    <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                        Complete a backup or wipe operation to generate certificates.
                    </p>
                    <button
                        className="btn btn-primary"
                        onClick={handleNewBackup}
                    >
                        🔍 Start Device Discovery
                    </button>
                </div>
            ) : (
                <div className="grid grid-cols-3 gap-6">
                    {/* Certificate List */}
                    <div className="col-span-2">
                        <div className="card">
                            <h3 className="font-semibold mb-4">Recent Certificates</h3>

                            <div className="space-y-3">
                                {certificates.map((cert) => (
                                    <div
                                        key={cert.id}
                                        className={`card certificate-card cursor-pointer transition-all ${selectedCert?.id === cert.id
                                            ? 'border-blue-500 bg-blue-50'
                                            : 'border-gray-200 hover:border-gray-300'
                                            }`}
                                        onClick={() => setSelectedCert(cert)}
                                        style={{
                                            padding: '1rem',
                                            border: selectedCert?.id === cert.id ? '2px solid #3b82f6' : '1px solid #e2e8f0',
                                            overflow: 'hidden'
                                        }}
                                    >
                                        <div className="flex justify-between items-start mb-3">
                                            <div className="flex items-center gap-2">
                                                <span style={{ fontSize: '1.5rem' }}>
                                                    {getCertTypeIcon(cert.type)}
                                                </span>
                                                <div>
                                                    <h4 className="font-semibold">{getCertTypeLabel(cert.type)}</h4>
                                                    <div className="text-xs" style={{ color: '#64748b' }}>
                                                        {formatDate(cert.created)}
                                                    </div>
                                                </div>
                                            </div>
                                            <div className="text-xs" style={{ color: '#64748b' }}>
                                                ID: {cert.id}
                                            </div>
                                        </div>

                                        <div className="space-y-2 mb-4">
                                            <FileLink
                                                path={cert.path}
                                                label="JSON Certificate"
                                                type="json"
                                            />
                                            {cert.pdfPath && (
                                                <FileLink
                                                    path={cert.pdfPath}
                                                    label="PDF Certificate"
                                                    type="pdf"
                                                />
                                            )}
                                        </div>

                                        <button
                                            className="btn btn-secondary text-sm"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                handleOpenPortalVerify(cert);
                                            }}
                                            style={{ width: '100%', marginTop: '0.75rem' }}
                                        >
                                            🌐 Open in Portal Verify
                                        </button>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>

                    {/* QR Preview and Details */}
                    <div>
                        {selectedCert ? (
                            <div className="card">
                                <h3 className="font-semibold mb-4">Certificate Details</h3>

                                <div className="mb-4">
                                    <div className="text-sm space-y-2">
                                        <div>
                                            <span className="font-medium">Type:</span>
                                            <div className="mt-1">{getCertTypeLabel(selectedCert.type)}</div>
                                        </div>
                                        <div>
                                            <span className="font-medium">Created:</span>
                                            <div className="mt-1">{formatDate(selectedCert.created)}</div>
                                        </div>
                                        <div>
                                            <span className="font-medium">Certificate ID:</span>
                                            <div className="mt-1 text-xs font-mono">{selectedCert.id}</div>
                                        </div>
                                    </div>
                                </div>

                                {selectedCert.verifyUrl && (
                                    <div className="mb-4">
                                        <QRPreview
                                            data={selectedCert.verifyUrl}
                                            title="Verification QR Code"
                                            size={150}
                                        />
                                    </div>
                                )}

                                <div className="space-y-2">
                                    <button
                                        className="btn btn-primary text-sm"
                                        onClick={() => handleOpenPortalVerify(selectedCert)}
                                        style={{ width: '100%' }}
                                    >
                                        🔍 Verify Online
                                    </button>

                                    <FileLink
                                        path={selectedCert.path}
                                        label="Open JSON"
                                        type="json"
                                    />

                                    {selectedCert.pdfPath && (
                                        <FileLink
                                            path={selectedCert.pdfPath}
                                            label="Open PDF"
                                            type="pdf"
                                        />
                                    )}
                                </div>

                                <div className="alert alert-info mt-4">
                                    <h4 className="font-semibold mb-2">📱 Mobile Verification</h4>
                                    <p className="text-xs">
                                        Scan the QR code with a mobile device to instantly verify this
                                        certificate's authenticity via the SecureWipe portal.
                                    </p>
                                </div>
                            </div>
                        ) : (
                            <div className="card text-center" style={{ padding: '2rem' }}>
                                <div style={{ fontSize: '3rem', marginBottom: '1rem', opacity: 0.3 }}>📄</div>
                                <p className="text-sm" style={{ color: '#64748b' }}>
                                    Select a certificate to view details and QR code.
                                </p>
                            </div>
                        )}
                    </div>
                </div>
            )}

            {/* Verification Instructions */}
            <div className="card mt-6">
                <h3 className="font-semibold mb-4">🔍 Verification Instructions</h3>
                <div className="grid grid-cols-2 gap-6 text-sm">
                    <div>
                        <h4 className="font-semibold mb-2">Command Line Verification</h4>
                        <div className="bg-gray-100 p-3 rounded font-mono text-xs">
                            <div>securewipe cert verify --file certificate.json</div>
                            <div>securewipe cert verify --qr-scan</div>
                        </div>
                        <p className="mt-2" style={{ color: '#64748b' }}>
                            Use the SecureWipe CLI to verify certificates offline or scan QR codes directly.
                        </p>
                    </div>
                    <div>
                        <h4 className="font-semibold mb-2">Web Portal Verification</h4>
                        <div className="bg-gray-100 p-3 rounded text-xs">
                            <div>1. Open: http://localhost:8000</div>
                            <div>2. Upload JSON certificate or scan QR</div>
                            <div>3. View verification results</div>
                        </div>
                        <p className="mt-2" style={{ color: '#64748b' }}>
                            Use the web portal for convenient verification and detailed analysis.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default Certificates;
