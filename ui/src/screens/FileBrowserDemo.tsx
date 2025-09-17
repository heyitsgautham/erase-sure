import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import ErrorBoundary from '../components/ErrorBoundary';
import FileBrowser from '../components/FileBrowser';

const FileBrowserDemo: React.FC = () => {
  const [selectedPaths] = useState<string[]>([]);
  const [testResult, setTestResult] = useState<string>('');

  const testTauriCommand = async () => {
    try {
      console.log('Testing browse_folders command...');
      const result = await invoke('browse_folders', { path: null });
      console.log('Command result:', result);
      setTestResult(JSON.stringify(result, null, 2));
    } catch (error) {
      console.error('Command failed:', error);
      setTestResult(`Error: ${error}`);
    }
  };

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto', padding: '2rem' }}>
      <h1 className="font-semibold mb-6" style={{ fontSize: '2rem' }}>
        File Browser Demo
      </h1>
      
      {/* Test button for debugging */}
      <div className="mb-6">
        <button
          className="btn btn-primary mb-4"
          onClick={testTauriCommand}
        >
          Test browse_folders Command
        </button>
        
        {testResult && (
          <div className="card">
            <h3 className="font-semibold mb-2">Test Result:</h3>
            <pre style={{ whiteSpace: 'pre-wrap', fontSize: '0.875rem' }}>
              {testResult}
            </pre>
          </div>
        )}
      </div>
      
      <div className="mb-6">
        <div className="card">
          <h3 className="font-semibold mb-4">File Browser Component</h3>
          <p style={{ marginBottom: '1rem', color: '#64748b' }}>
            This should load the file browser below. If you see this text but no file browser,
            there may be an error in the component.
          </p>
          
          <ErrorBoundary>
            <FileBrowser
              onSelectionChange={(paths) => console.log('Selected:', paths)}
              multiSelect={true}
              allowFiles={true}
              allowFolders={true}
              maxSelectionSize={100 * 1024 * 1024}
              title="Browse Files and Folders"
            />
          </ErrorBoundary>
        </div>
      </div>

      {selectedPaths.length > 0 && (
        <div className="card">
          <h3 className="font-semibold mb-4">Selected Paths:</h3>
          <div className="space-y-2">
            {selectedPaths.map((path, index) => (
              <div key={index} className="text-sm bg-gray-50 p-2 rounded">
                {path}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default FileBrowserDemo;
