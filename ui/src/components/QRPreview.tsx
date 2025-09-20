import { useEffect, useState } from 'react';

interface QRPreviewProps {
    data: string;
    title?: string;
    size?: number;
}

function QRPreview({ data, title, size = 200 }: QRPreviewProps) {
    const [qrDataUrl, setQrDataUrl] = useState<string>('');

    useEffect(() => {
        generateQRCode(data, size).then(setQrDataUrl).catch(console.error);
    }, [data, size]);

    return (
        <div className="qr-preview">
            {title && <h4 className="font-medium mb-2">{title}</h4>}
            {qrDataUrl ? (
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
                    Generating QR...
                </div>
            )}
            <div className="text-xs text-center mt-2" style={{ maxWidth: size, wordBreak: 'break-all' }}>
                {data.length > 50 ? `${data.substring(0, 50)}...` : data}
            </div>
        </div>
    );
}

// Generate QR code using the qrcode library
async function generateQRCode(text: string, size: number): Promise<string> {
    try {
        // Import QRCode dynamically to avoid SSR issues
        const QRCode = await import('qrcode');
        
        return await QRCode.toDataURL(text, {
            width: size,
            margin: 2,
            color: {
                dark: '#000000',
                light: '#FFFFFF'
            },
            errorCorrectionLevel: 'M'
        });
    } catch (error) {
        console.error('Failed to generate QR code:', error);
        
        // Fallback to placeholder if QR generation fails
        return new Promise((resolve) => {
            const svg = `
        <svg width="${size}" height="${size}" xmlns="http://www.w3.org/2000/svg">
          <rect width="100%" height="100%" fill="white"/>
          <rect x="10%" y="10%" width="10%" height="10%" fill="black"/>
          <rect x="80%" y="10%" width="10%" height="10%" fill="black"/>
          <rect x="10%" y="80%" width="10%" height="10%" fill="black"/>
          <text x="50%" y="50%" text-anchor="middle" dy="0.3em" font-size="12" fill="black">QR</text>
        </svg>
      `;
            const dataUrl = `data:image/svg+xml;base64,${btoa(svg)}`;
            resolve(dataUrl);
        });
    }
}

export default QRPreview;
