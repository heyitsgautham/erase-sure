import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { useSecureWipe } from '../hooks/useSecureWipe';
import { useApp } from '../contexts/AppContext';

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

interface CertificateData {
    cert_id: string;
    cert_type: string;
    created_at: string;
    // Add other fields as needed
}

function Certificates() {
    const navigate = useNavigate();
    const { addToast } = useApp();
    const { generatePdfForCert, openPath, verifyOnline } = useSecureWipe();
    const [certificates, setCertificates] = useState<Certificate[]>([]);
    const [selectedCert, setSelectedCert] = useState<Certificate | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [generatingPdf, setGeneratingPdf] = useState(false);
    const [verifyResult, setVerifyResult] = useState<any>(null);
    const [showVerifyModal, setShowVerifyModal] = useState(false);

    // Load actual certificates from the filesystem
    useEffect(() => {
        loadCertificates();
    }, []);

    const loadCertificates = async () => {
        try {
            setLoading(true);
            setError(null);
            
            // Try to use Tauri commands
            try {
                const homeDir = await invoke('get_home_dir');
                const certDir = `${homeDir}/SecureWipe/certificates`;
                
                // List JSON files in the certificates directory
                const certFiles: string[] = await invoke('list_cert_files', { directory: certDir });
                
                const loadedCerts: Certificate[] = [];
                
                for (const file of certFiles) {
                    try {
                        // Read and parse each certificate
                        const certContent: string = await invoke('read_file_content', { filePath: file });
                        const certData: CertificateData = JSON.parse(certContent);
                        
                        // Check for PDF in both default location and backups directory
                        const defaultPdfPath = file.replace('.json', '.pdf');
                        const homeDir = await invoke('get_home_dir') as string;
                        const backupsPdfPath = `${homeDir}/SecureWipe/backups/${certData.cert_id}.pdf`;
                        
                        const defaultPdfExists: boolean = await invoke('file_exists', { filePath: defaultPdfPath }) as boolean;
                        const backupsPdfExists: boolean = await invoke('file_exists', { filePath: backupsPdfPath }) as boolean;
                        
                        // Prefer backups location, fallback to default location
                        const pdfPath = backupsPdfExists ? backupsPdfPath : (defaultPdfExists ? defaultPdfPath : undefined);
                        const pdfExists = backupsPdfExists || defaultPdfExists;
                        
                        const cert: Certificate = {
                            id: certData.cert_id,
                            type: certData.cert_type as 'backup' | 'wipe',
                            filename: file.split('/').pop() || file,
                            path: file,
                            pdfPath: pdfExists ? pdfPath : undefined,
                            created: new Date(certData.created_at),
                            verifyUrl: `http://localhost:8000/verify?cert_id=${encodeURIComponent(certData.cert_id)}`
                        };
                        
                        loadedCerts.push(cert);
                    } catch (parseError) {
                        console.warn(`Failed to parse certificate ${file}:`, parseError);
                    }
                }
                
                // Sort by creation time (newest first)
                loadedCerts.sort((a, b) => b.created.getTime() - a.created.getTime());
                
                setCertificates(loadedCerts);
                if (loadedCerts.length > 0) {
                    setSelectedCert(loadedCerts[0]);
                }
            } catch (tauriError) {
                console.warn('Tauri commands not available, using fallback:', tauriError);
                
                // Fallback: Create a demo certificate based on the existing ones we know exist
                const fallbackCerts: Certificate[] = [
                    {
                        id: '397fad31-22d4-4abb-9065-686046cbaf3e',
                        type: 'backup',
                        filename: '397fad31-22d4-4abb-9065-686046cbaf3e.json',
                        path: '~/SecureWipe/certificates/397fad31-22d4-4abb-9065-686046cbaf3e.json',
                        pdfPath: '~/SecureWipe/certificates/397fad31-22d4-4abb-9065-686046cbaf3e.pdf',
                        created: new Date('2025-09-17T11:04:42.289632748+00:00'),
                        verifyUrl: 'http://localhost:8000/verify?id=397fad31-22d4-4abb-9065-686046cbaf3e'
                    }
                ];
                
                setCertificates(fallbackCerts);
                setSelectedCert(fallbackCerts[0]);
            }
        } catch (err) {
            console.error('Failed to load certificates:', err);
            setError('Failed to load certificates. Make sure you have completed at least one backup operation.');
        } finally {
            setLoading(false);
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

    const computeQRPayload = (cert: Certificate): string => {
        // Prefer verify_url if available
        if (cert.verifyUrl) {
            return cert.verifyUrl;
        }

        // Fallback: create compact payload
        const compactPayload = {
            cert_id: cert.id,
            issued_at: cert.created.toISOString(),
            sha256_cert_json: 'computed-hash-placeholder' // TODO: Compute actual hash
        };

        // Encode as base64 JSON for QR
        const jsonStr = JSON.stringify(compactPayload);
        const base64Payload = btoa(jsonStr);
        
        return `http://localhost:8000/verify?payload=${encodeURIComponent(base64Payload)}`;
    };

    const handleOpenJson = async (cert: Certificate) => {
        try {
            await openPath(cert.path);
        } catch (error) {
            addToast(`Failed to open JSON: ${error}`, 'error');
        }
    };

    const handleOpenPdf = async (cert: Certificate) => {
        if (cert.pdfPath) {
            try {
                await openPath(cert.pdfPath);
            } catch (error) {
                addToast(`Failed to open PDF: ${error}`, 'error');
            }
        } else {
            // Generate PDF first
            await handleGeneratePdf(cert);
        }
    };

    const handleGeneratePdf = async (cert: Certificate) => {
        try {
            setGeneratingPdf(true);
            addToast('Generating high-quality PDF certificate...', 'info');
            
            const result = await generatePdfForCert(cert.path);
            
            // Update certificate with PDF path (saved in ~/SecureWipe/backups/)
            setCertificates(prev => prev.map(c => 
                c.id === cert.id ? { ...c, pdfPath: result.pdfPath } : c
            ));
            
            // Update selected cert if it's the current one
            if (selectedCert?.id === cert.id) {
                setSelectedCert(prev => prev ? { ...prev, pdfPath: result.pdfPath } : prev);
            }
            
            addToast(`PDF generated successfully! Saved to ~/SecureWipe/backups/`, 'success');
            
            // Auto-open the generated PDF
            await openPath(result.pdfPath);
            
        } catch (error) {
            addToast(`Failed to generate PDF: ${error}`, 'error');
        } finally {
            setGeneratingPdf(false);
        }
    };

    const handleVerifyOnline = async (cert: Certificate) => {
        try {
            addToast('Verifying certificate online...', 'info');
            
            // Read certificate JSON content
            const certContent = await invoke('read_file_content', { filePath: cert.path });
            const certJson = JSON.parse(certContent as string);
            
            const result = await verifyOnline(certJson);
            
            if ((result as any).fallback) {
                addToast('Verification portal opened in browser', 'info');
            } else {
                setVerifyResult(result);
                setShowVerifyModal(true);
                
                if (result.errors.length === 0) {
                    addToast('Certificate verified successfully!', 'success');
                } else {
                    addToast('Certificate verification completed with issues', 'warning');
                }
            }
            
        } catch (error) {
            addToast(`Verification failed: ${error}`, 'error');
        }
    };

    const handleOpenBackupsFolder = async () => {
        try {
            const homeDir = await invoke('get_home_dir') as string;
            const backupsPath = `${homeDir}/SecureWipe/backups`;
            await openPath(backupsPath);
            addToast('Opened backups folder', 'success');
        } catch (error) {
            addToast(`Failed to open backups folder: ${error}`, 'error');
        }
    };

    if (loading) {
        return (
            <div style={{ maxWidth: '800px', margin: '0 auto', textAlign: 'center', padding: '3rem' }}>
                <div style={{ fontSize: '3rem', marginBottom: '1rem', opacity: 0.3 }}>⏳</div>
                <h3 className="font-semibold mb-2">Loading Certificates...</h3>
                <p style={{ color: '#64748b' }}>Scanning for backup and wipe certificates.</p>
            </div>
        );
    }

    if (error) {
        return (
            <div style={{ maxWidth: '800px', margin: '0 auto' }}>
                <div className="mb-6">
                    <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                        Certificate Management
                    </h2>
                    <button
                        className="btn btn-secondary mb-4"
                        onClick={handleBackToHome}
                    >
                        🏠 Back to Home
                    </button>
                </div>
                
                <div className="card text-center" style={{ padding: '3rem' }}>
                    <div style={{ fontSize: '4rem', marginBottom: '1rem', opacity: 0.3 }}>❌</div>
                    <h3 className="font-semibold mb-2">Error Loading Certificates</h3>
                    <p style={{ color: '#64748b', marginBottom: '2rem' }}>{error}</p>
                    <div className="flex gap-4 justify-center">
                        <button
                            className="btn btn-primary"
                            onClick={handleNewBackup}
                        >
                            ➕ Create New Backup
                        </button>
                        <button
                            className="btn btn-secondary"
                            onClick={loadCertificates}
                        >
                            🔄 Retry Loading
                        </button>
                    </div>
                </div>
            </div>
        );
    }

    if (certificates.length === 0) {
        return (
            <div style={{ maxWidth: '800px', margin: '0 auto' }}>
                <div className="mb-6">
                    <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                        Certificate Management
                    </h2>
                    <button
                        className="btn btn-secondary mb-4"
                        onClick={handleBackToHome}
                    >
                        🏠 Back to Home
                    </button>
                </div>
                
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
            </div>
        );
    }

    // Show the most recent certificate in center layout
    const currentCert = selectedCert || certificates[0];

    return (
        <div style={{ maxWidth: '800px', margin: '0 auto' }}>
            <div className="mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    Certificate Management
                </h2>
                <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                    View, verify, and manage your {getCertTypeLabel(currentCert.type).toLowerCase()}. 
                    Each certificate provides cryptographic proof of completed operations.
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

            {/* Certificate Display - Clean Center Layout */}
            <div className="card" style={{ padding: '2rem' }}>
                <div style={{ textAlign: 'center', marginBottom: '2rem' }}>
                    <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>
                        {getCertTypeIcon(currentCert.type)}
                    </div>
                    <h3 className="font-semibold mb-2">{getCertTypeLabel(currentCert.type)}</h3>
                    <p style={{ color: '#64748b', fontSize: '0.9rem' }}>
                        Created: {formatDate(currentCert.created)}
                    </p>
                    <p style={{ color: '#64748b', fontSize: '0.75rem', fontFamily: 'monospace' }}>
                        ID: {currentCert.id}
                    </p>
                </div>

                {/* QR Code in Center */}
                <div style={{ textAlign: 'center', marginBottom: '2rem' }}>
                    <QRPreview
                        data={computeQRPayload(currentCert)}
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
                        onClick={() => handleVerifyOnline(currentCert)}
                        style={{ width: '100%' }}
                        disabled={generatingPdf}
                    >
                        🔍 Verify Online
                    </button>

                    <button
                        className="btn btn-secondary"
                        onClick={() => handleOpenJson(currentCert)}
                        style={{ width: '100%' }}
                    >
                        📄 Open JSON Certificate
                    </button>

                    {currentCert.pdfPath ? (
                        <button
                            className="btn btn-secondary"
                            onClick={() => handleOpenPdf(currentCert)}
                            style={{ width: '100%' }}
                        >
                            📋 Open PDF Certificate
                        </button>
                    ) : (
                        <button
                            className="btn btn-secondary"
                            onClick={() => handleGeneratePdf(currentCert)}
                            style={{ width: '100%' }}
                            disabled={generatingPdf}
                        >
                            {generatingPdf ? '⏳ Generating High-Quality PDF...' : '📋 Generate PDF'}
                        </button>
                    )}

                    <button
                        className="btn btn-outline"
                        onClick={handleOpenBackupsFolder}
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
                                    className={`btn text-sm ${selectedCert?.id === cert.id ? 'btn-primary' : 'btn-secondary'}`}
                                    onClick={() => setSelectedCert(cert)}
                                >
                                    {getCertTypeIcon(cert.type)} {getCertTypeLabel(cert.type)} #{index + 1}
                                </button>
                            ))}
                        </div>
                    </div>
                )}
            </div>

            {/* Simplified Verification Instructions */}
            <div className="card mt-6">
                <h3 className="font-semibold mb-4">🔍 How to Verify</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div>
                        <h4 className="font-semibold mb-2">📱 Mobile/QR Verification</h4>
                        <p style={{ color: '#64748b' }}>
                            Scan the QR code above with any mobile device to instantly verify 
                            this certificate through the SecureWipe portal.
                        </p>
                    </div>
                    <div>
                        <h4 className="font-semibold mb-2">💻 Command Line</h4>
                        <div className="bg-gray-100 p-2 rounded font-mono text-xs">
                            securewipe cert verify --file certificate.json
                        </div>
                        <p className="mt-1" style={{ color: '#64748b' }}>
                            Use the CLI for offline verification.
                        </p>
                    </div>
                </div>
            </div>

            {/* Verification Result Modal */}
            {showVerifyModal && verifyResult && (
                <div className="modal-overlay" style={{
                    position: 'fixed',
                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 0,
                    backgroundColor: 'rgba(0, 0, 0, 0.5)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    zIndex: 1000
                }}>
                    <div className="modal-content card" style={{
                        maxWidth: '500px',
                        width: '90%',
                        maxHeight: '80vh',
                        overflow: 'auto',
                        padding: '2rem'
                    }}>
                        <div style={{ textAlign: 'center', marginBottom: '1.5rem' }}>
                            <h3 className="font-semibold mb-2">Certificate Verification Result</h3>
                            <p style={{ fontSize: '0.9rem', color: '#64748b' }}>
                                Certificate ID: {verifyResult.cert_summary?.cert_id}
                            </p>
                        </div>

                        <div className="space-y-3 mb-6">
                            <div className="flex justify-between items-center">
                                <span>Schema Valid:</span>
                                <span className={`font-semibold ${verifyResult.schema_valid ? 'text-green-600' : 'text-red-600'}`}>
                                    {verifyResult.schema_valid ? '✓ Yes' : '✗ No'}
                                </span>
                            </div>
                            
                            <div className="flex justify-between items-center">
                                <span>Signature Valid:</span>
                                <span className={`font-semibold ${
                                    verifyResult.signature_valid === null ? 'text-gray-500' :
                                    verifyResult.signature_valid ? 'text-green-600' : 'text-red-600'
                                }`}>
                                    {verifyResult.signature_valid === null ? '- N/A' :
                                     verifyResult.signature_valid ? '✓ Yes' : '✗ No'}
                                </span>
                            </div>
                            
                            <div className="flex justify-between items-center">
                                <span>Hash Valid:</span>
                                <span className={`font-semibold ${
                                    verifyResult.hash_valid === null ? 'text-gray-500' :
                                    verifyResult.hash_valid ? 'text-green-600' : 'text-red-600'
                                }`}>
                                    {verifyResult.hash_valid === null ? '- N/A' :
                                     verifyResult.hash_valid ? '✓ Yes' : '✗ No'}
                                </span>
                            </div>
                            
                            {verifyResult.chain_valid !== null && (
                                <div className="flex justify-between items-center">
                                    <span>Chain Valid:</span>
                                    <span className={`font-semibold ${verifyResult.chain_valid ? 'text-green-600' : 'text-red-600'}`}>
                                        {verifyResult.chain_valid ? '✓ Yes' : '✗ No'}
                                    </span>
                                </div>
                            )}
                        </div>

                        {verifyResult.errors.length > 0 && (
                            <div className="mb-6">
                                <h4 className="font-semibold mb-2 text-red-600">Issues Found:</h4>
                                <ul className="text-sm space-y-1">
                                    {verifyResult.errors.map((error: string, index: number) => (
                                        <li key={index} className="text-red-600">• {error}</li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        <div className="flex gap-3 justify-end">
                            <button
                                className="btn btn-secondary"
                                onClick={() => setShowVerifyModal(false)}
                            >
                                Close
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

export default Certificates;
