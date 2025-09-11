import { useEffect, useRef } from 'react';

interface LogViewerProps {
  logs: string[];
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
    const logText = logs.join('\n');
    navigator.clipboard.writeText(logText).catch(console.error);
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
            <div key={index}>
              {log}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

export default LogViewer;
