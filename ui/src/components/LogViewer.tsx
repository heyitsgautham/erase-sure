import { useEffect, useRef } from 'react';
import { useSecureWipe } from '../hooks/useSecureWipe';

interface LogViewerProps {
    logs?: string[]; // Legacy prop for compatibility
    title?: string;
}

function LogViewer({ logs: legacyLogs, title = "Operation Logs" }: LogViewerProps) {
    const logRef = useRef<HTMLDivElement>(null);
    const { logs: streamLogs } = useSecureWipe();

    // Use stream logs if available, otherwise fall back to legacy logs
    const displayLogs = streamLogs.length > 0 ? streamLogs : [];
    const fallbackLogs = legacyLogs || [];

    useEffect(() => {
        if (logRef.current) {
            logRef.current.scrollTop = logRef.current.scrollHeight;
        }
    }, [displayLogs, fallbackLogs]);

    const copyLogs = () => {
        let logText = '';
        if (displayLogs.length > 0) {
            logText = displayLogs.map(log => `[${log.ts}] ${log.stream.toUpperCase()}: ${log.line}`).join('\n');
        } else {
            logText = fallbackLogs.join('\n');
        }
        navigator.clipboard.writeText(logText).catch(console.error);
    };

    const hasLogs = displayLogs.length > 0 || fallbackLogs.length > 0;

    return (
        <div className="card">
            <div className="flex justify-between items-center mb-4">
                <h3 className="font-semibold">{title}</h3>
                <button
                    className="btn btn-secondary btn-sm"
                    onClick={copyLogs}
                    disabled={!hasLogs}
                >
                    Copy Logs
                </button>
            </div>
            <div
                ref={logRef}
                className="log-viewer"
            >
                {!hasLogs ? (
                    <div style={{ color: '#94a3b8', fontStyle: 'italic' }}>
                        No logs yet...
                    </div>
                ) : (
                    <>
                        {displayLogs.map((log, index) => (
                            <div key={index} className={`log-entry log-${log.stream}`}>
                                <span className="log-timestamp">[{new Date(log.ts).toLocaleTimeString()}]</span>
                                <span className={`log-stream log-stream-${log.stream}`}>{log.stream.toUpperCase()}:</span>
                                <span className="log-message">{log.line}</span>
                            </div>
                        ))}
                        {fallbackLogs.map((log, index) => (
                            <div key={`fallback-${index}`} className="log-entry">
                                {log}
                            </div>
                        ))}
                    </>
                )}
            </div>
        </div>
    );
}

export default LogViewer;
