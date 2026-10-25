/** The two ways to check a certificate: the QR code and the CLI. */
function HowToVerify() {
    return (
        <div className="card mt-6">
            <h3 className="font-semibold mb-4">🔍 How to Verify</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div>
                    <h4 className="font-semibold mb-2">📱 Mobile/QR Verification</h4>
                    <p style={{ color: '#64748b' }}>
                        Scan the QR code above with any mobile device to instantly verify this certificate through the
                        SecureWipe portal.
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
    );
}

export default HowToVerify;
