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
        const logText = logs.map(log => `[${formatTimestamp(log.ts)}] ${log.stream}: ${log.line}`).join('\n');
        navigator.clipboard.writeText(logText).catch(console.error);
    };

    const formatTimestamp = (ts: string) => {
        try {
            return new Date(ts).toLocaleTimeString();
        } catch {
            return ts;
        }
    };

    return (
        <div className="card">
            <div className="flex justify-between items-center mb-4">
                <h3 className="font-semibold">{title}</h3>
                <button
                    className="btn btn-secondary btn-sm"
                    onClick={copyLogs}
                    disabled={logs.length === 0}
                >
                    Copy Logs
                </button>
            </div>
            <div
                ref={logRef}
                className="log-viewer"
            >
                {logs.length === 0 ? (
                    <div style={{ color: '#94a3b8', fontStyle: 'italic' }}>
                        No logs yet...
                    </div>
                ) : (
                    logs.map((log, index) => (
                        <div key={index} className="flex gap-2 mb-1">
                            <span className="text-gray-400 text-xs shrink-0 w-20">
                                {formatTimestamp(log.ts)}
                            </span>
                            <span className={`text-xs shrink-0 w-12 ${log.stream === 'stderr' ? 'text-red-400' : 'text-blue-400'
                                }`}>
                                {log.stream}
                            </span>
                            <span className={`flex-1 ${log.stream === 'stderr' ? 'text-red-300' : 'text-green-300'
                                }`}>
                                {log.line}
                            </span>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
}

export default LogViewer;
