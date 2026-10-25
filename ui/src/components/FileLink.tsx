import { useSecureWipe } from '../hooks/useSecureWipe';
import { createLogger } from '../lib/logger';

const log = createLogger('file-link');

interface FileLinkProps {
    path: string;
    label?: string;
    type?: 'json' | 'pdf' | 'folder';
}

function FileLink({ path, label, type = 'folder' }: FileLinkProps) {
    const { openPath } = useSecureWipe();

    const openFile = async () => {
        try {
            await openPath(path);
        } catch (error) {
            // openPath already told the user; keep the detail for debugging.
            log.error('opening path failed', { path }, error);
        }
    };

    const getIcon = () => {
        switch (type) {
            case 'json':
                return '📄';
            case 'pdf':
                return '📋';
            case 'folder':
            default:
                return '📁';
        }
    };

    return (
        <button
            className="btn btn-secondary"
            onClick={openFile}
            style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
                textAlign: 'left',
                width: '100%', // Changed from minWidth to full width
                minHeight: '2.5rem' // Ensure consistent height
            }}
        >
            <span>{getIcon()}</span>
            <div style={{ flex: 1, minWidth: 0, overflow: 'hidden' }}>
                <div className="font-medium" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {label || path}
                </div>
                {label && (
                    <div className="text-xs" style={{
                        opacity: 0.7,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap'
                    }}>
                        {path}
                    </div>
                )}
            </div>
        </button>
    );
}

export default FileLink;
