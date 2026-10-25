import { useEffect, useState } from 'react';
import { createLogger } from '../lib/logger';

const log = createLogger('qr-preview');

interface QRPreviewProps {
    data: string;
    title?: string;
    size?: number;
}

function QRPreview({ data, title, size = 200 }: QRPreviewProps) {
    const [qrDataUrl, setQrDataUrl] = useState<string>('');
    const [failed, setFailed] = useState(false);

    useEffect(() => {
        let current = true;
        setFailed(false);
        generateQRCode(data, size)
            .then((url) => {
                if (current) setQrDataUrl(url);
            })
            .catch((error) => {
                // A placeholder that looks like a QR code would send people to scan nothing.
                log.error('QR code generation failed', undefined, error);
                if (current) setFailed(true);
            });
        return () => {
            current = false;
        };
    }, [data, size]);

    return (
        <div className="qr-preview">
            {title && <h4 className="font-medium mb-2">{title}</h4>}
            {qrDataUrl && !failed ? (
                <img
                    src={qrDataUrl}
                    alt="QR Code"
                    width={size}
                    height={size}
                    style={{ border: '1px solid #e2e8f0', borderRadius: '4px' }}
                />
            ) : (
                <div
                    style={{
                        width: size,
                        height: size,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        backgroundColor: '#f1f5f9',
                        border: '1px solid #e2e8f0',
                        borderRadius: '4px'
                    }}
                >
                    {failed ? 'QR code unavailable' : 'Generating QR...'}
                </div>
            )}
            <div className="text-xs text-center mt-2" style={{ maxWidth: size, wordBreak: 'break-all' }}>
                {data.length > 50 ? `${data.substring(0, 50)}...` : data}
            </div>
        </div>
    );
}

/** Render `text` as a PNG data URL. qrcode is loaded on first use to keep it out of the main bundle. */
async function generateQRCode(text: string, size: number): Promise<string> {
    const QRCode = await import('qrcode');
    return QRCode.toDataURL(text, {
        width: size,
        margin: 2,
        color: {
            dark: '#000000',
            light: '#FFFFFF'
        },
        errorCorrectionLevel: 'M'
    });
}

export default QRPreview;
