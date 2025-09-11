import { useNavigate } from 'react-router-dom';

function Home() {
    const navigate = useNavigate();

    const handleBackupAndWipe = () => {
        navigate('/discover');
    };

    const handleWipePlanOnly = () => {
        navigate('/wipe-plan');
    };

    return (
        <div className="text-center" style={{ maxWidth: '800px', margin: '0 auto', padding: '2rem 0' }}>
            {/* SIH Banner */}
            <div className="card mb-6" style={{ background: 'linear-gradient(135deg, #3b82f6 0%, #1e40af 100%)', color: 'white' }}>
                <h1 style={{ fontSize: '2rem', fontWeight: 'bold', marginBottom: '1rem' }}>
                    SecureWipe
                </h1>
                <p style={{ fontSize: '1.125rem', opacity: 0.9 }}>
                    Smart India Hackathon 2024 - Secure Data Destruction Solution
                </p>
                <p style={{ marginTop: '1rem', opacity: 0.8 }}>
                    Professional-grade disk wiping with cryptographic verification and compliance reporting
                </p>
            </div>

            {/* Mission Statement */}
            <div className="card mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    Mission Statement
                </h2>
                <p style={{ fontSize: '1.125rem', lineHeight: '1.6', color: '#475569' }}>
                    Ensuring complete data destruction with verifiable compliance for government agencies,
                    enterprises, and security-conscious organizations through advanced wiping algorithms
                    and tamper-proof certification.
                </p>
            </div>

            {/* Action Buttons */}
            <div className="grid grid-cols-1 gap-6" style={{ maxWidth: '600px', margin: '0 auto' }}>
                <button
                    className="btn btn-primary btn-large"
                    onClick={handleBackupAndWipe}
                    style={{
                        padding: '1.5rem 2rem',
                        fontSize: '1.25rem',
                        boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)'
                    }}
                >
                    🛡️ Backup & Wipe (Recommended)
                    <div style={{ fontSize: '0.875rem', opacity: 0.9, marginTop: '0.5rem' }}>
                        Complete workflow: Device discovery → Encrypted backup → Secure wipe → Certificates
                    </div>
                </button>

                <button
                    className="btn btn-secondary btn-large"
                    onClick={handleWipePlanOnly}
                    style={{
                        padding: '1.5rem 2rem',
                        fontSize: '1.25rem',
                        boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)'
                    }}
                >
                    📋 Wipe Plan Only (Safe Preview)
                    <div style={{ fontSize: '0.875rem', opacity: 0.7, marginTop: '0.5rem' }}>
                        Non-destructive planning and verification preview (MVP safe mode)
                    </div>
                </button>
            </div>

            {/* Feature Highlights */}
            <div className="grid grid-cols-3 gap-4 mt-8">
                <div className="card text-center">
                    <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>🔐</div>
                    <h3 className="font-semibold mb-2">AES-256 Encryption</h3>
                    <p className="text-sm" style={{ color: '#64748b' }}>
                        Military-grade backup encryption with integrity verification
                    </p>
                </div>

                <div className="card text-center">
                    <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>📱</div>
                    <h3 className="font-semibold mb-2">QR Verification</h3>
                    <p className="text-sm" style={{ color: '#64748b' }}>
                        Instant certificate validation via mobile portal
                    </p>
                </div>

                <div className="card text-center">
                    <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>⚡</div>
                    <h3 className="font-semibold mb-2">Multiple Standards</h3>
                    <p className="text-sm" style={{ color: '#64748b' }}>
                        DoD 5220.22-M, NIST SP 800-88, and custom algorithms
                    </p>
                </div>
            </div>

            {/* Safety Notice for MVP */}
            <div className="alert alert-info mt-6" style={{ textAlign: 'left' }}>
                <h4 className="font-semibold mb-2">🛡️ MVP Safety Mode</h4>
                <p className="text-sm">
                    This demonstration version operates in safe mode - no actual disk wiping will occur.
                    All operations are simulated for evaluation purposes while maintaining full workflow validation.
                </p>
            </div>
        </div>
    );
}

export default Home;
