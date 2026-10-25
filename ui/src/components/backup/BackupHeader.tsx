const SECURITY_FEATURES = [
    'AES-256-CTR Encryption',
    'PBKDF2 Key Derivation',
    'HMAC-SHA256 Integrity',
    'N=5 Sample Verification',
];

const checkBadge = {
    width: '24px',
    height: '24px',
    background: '#10b981',
    borderRadius: '50%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '12px',
    color: 'white',
} as const;

/** Title and the list of protections every backup gets. */
function BackupHeader() {
    return (
        <div className="mb-8">
            <div className="flex items-center gap-4 mb-4">
                <div style={{ 
                    width: '56px', 
                    height: '56px', 
                    background: 'linear-gradient(135deg, #3b82f6 0%, #1e40af 100%)', 
                    borderRadius: '16px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '28px',
                    boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)'
                }}>
                    🛡️
                </div>
                <div>
                    <h1 className="font-semibold" style={{ fontSize: '2.25rem', marginBottom: '0.5rem', color: '#1e293b' }}>
                        Encrypted Backup
                    </h1>
                    <p style={{ color: '#64748b', fontSize: '1.125rem' }}>
                        Secure your data with military-grade AES-256-CTR encryption
                    </p>
                </div>
            </div>
            
            {/* Security Features Card */}
            <div className="alert alert-info" style={{ 
                border: 'none', 
                background: 'linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%)',
                borderRadius: '16px',
                padding: '1.5rem'
            }}>
                <div className="flex items-start gap-4">
                    <div style={{ 
                        fontSize: '2rem',
                        background: 'linear-gradient(135deg, #3b82f6 0%, #1e40af 100%)',
                        width: '48px',
                        height: '48px',
                        borderRadius: '12px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center'
                    }}>
                        🔐
                    </div>
                    <div className="flex-1">
                        <h3 className="font-semibold mb-3" style={{ color: '#1e40af', fontSize: '1.25rem' }}>
                            Advanced Security Features
                        </h3>
                        <div className="grid grid-cols-2 gap-4">
                            {SECURITY_FEATURES.map((feature) => (
                                <div key={feature} className="flex items-center gap-3">
                                    <div style={checkBadge}>✓</div>
                                    <span style={{ fontWeight: '500' }}>{feature}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default BackupHeader;
