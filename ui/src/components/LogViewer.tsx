import { useEffect, useRef } from 'react';
import type { LogEvent } from '../types/securewipe';

interface LogViewerProps {
    logs: LogEvent[];
    title?: string;
}

function LogViewer({ logs, title = "Operation Logs" }: LogViewerProps) {
    const logRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (logRef.current) {
            logRef.current.scrollTop = logRef.current.scrollHeight;
        }
    }, [logs]);

    const copyLogs = () => {
        const logText = logs.map(log => `[${log.ts}] ${log.stream.toUpperCase()}: ${log.line}`).join('\n');
        navigator.clipboard.writeText(logText).catch(() => {
            // Clipboard access can be denied; the logs stay on screen either way.
        });
    };

    const hasLogs = logs.length > 0;

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
                    logs.map((log, index) => (
                        <div key={index} className={`log-entry log-${log.stream}`}>
                            <span className="log-timestamp">[{new Date(log.ts).toLocaleTimeString()}]</span>
                            <span className={`log-stream log-stream-${log.stream}`}>{log.stream.toUpperCase()}:</span>
                            <span className="log-message">{log.line}</span>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
}

export default LogViewer;
