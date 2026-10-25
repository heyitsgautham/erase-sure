import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands } from '../lib/commands';
import { useSecureWipe } from '../hooks/useSecureWipe';
import type { VerifyResult } from '../types/securewipe';
import { useApp } from '../contexts/AppContext';

import CertificateCard from '../components/certificates/CertificateCard';
import HowToVerify from '../components/certificates/HowToVerify';
import VerifyResultModal from '../components/certificates/VerifyResultModal';
import { useCertificates } from '../hooks/useCertificates';
import { certTypeLabel } from '../lib/certificates';
import type { Certificate } from '../lib/certificates';
import { describeError } from '../lib/logger';

function Certificates() {
    const navigate = useNavigate();
    const { addToast } = useApp();
    const { generatePdfForCert, openPath, verifyOnline } = useSecureWipe();
    const { certificates, loading, error, reload, setPdfPath } = useCertificates();
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [generatingPdf, setGeneratingPdf] = useState(false);
    const [verifyResult, setVerifyResult] = useState<VerifyResult | null>(null);
    const [showVerifyModal, setShowVerifyModal] = useState(false);
    const selectedCert = certificates.find((cert) => cert.id === selectedId) ?? null;

    const handleBackToHome = () => {
        navigate('/');
    };

    const handleNewBackup = () => {
        navigate('/discover');
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
            
            setPdfPath(cert.id, result.pdfPath);

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

            const certJson = JSON.parse(await commands.readFileContent(cert.path)) as Record<string, unknown>;
            const outcome = await verifyOnline(certJson);

            if (outcome.kind === 'opened-portal') {
                addToast('Verification portal opened in browser', 'info');
                return;
            }
            setVerifyResult(outcome.result);
            setShowVerifyModal(true);
            if (outcome.result.errors.length === 0) {
                addToast('Certificate verified successfully!', 'success');
            } else {
                addToast('Certificate verification completed with issues', 'warning');
            }
        } catch (error) {
            addToast(`Verification failed: ${describeError(error).message}`, 'error');
        }
    };

    const handleOpenBackupsFolder = async () => {
        try {
            const homeDir = await commands.getHomeDir();
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
                            onClick={reload}
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
                    View, verify, and manage your {certTypeLabel(currentCert.type).toLowerCase()}. 
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

            <CertificateCard
                certificate={currentCert}
                certificates={certificates}
                generatingPdf={generatingPdf}
                onVerify={handleVerifyOnline}
                onOpenJson={handleOpenJson}
                onOpenPdf={handleOpenPdf}
                onGeneratePdf={handleGeneratePdf}
                onOpenBackupsFolder={handleOpenBackupsFolder}
                onSelect={setSelectedId}
            />

            <HowToVerify />

            {showVerifyModal && verifyResult && (
                <VerifyResultModal result={verifyResult} onClose={() => setShowVerifyModal(false)} />
            )}
        </div>
    );
}

export default Certificates;
