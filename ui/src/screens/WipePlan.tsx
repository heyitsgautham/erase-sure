import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe } from '../hooks/useSecureWipeCompat';
import LogViewer from '../components/LogViewer';

function WipePlan() {
    const navigate = useNavigate();
    const { state, addToast } = useApp();
    const { createWipePlan, logs } = useSecureWipe();
    const [showJsonView, setShowJsonView] = useState(false);

    const handleCreatePlan = async () => {
        if (!state.selectedDevice) {
            addToast('Please select a device first', 'error');
            return;
        }

        try {
            addToast(`Analyzing ${state.selectedDevice.model} for wipe strategy...`, 'info');
            await createWipePlan(state.selectedDevice.path);
            addToast('Wipe plan created successfully! Safe preview mode enabled.', 'success');
        } catch (error) {
            console.error('Failed to create wipe plan:', error);
            addToast('Failed to create wipe plan. Please check device connection.', 'error');
        }
    };

    const handleContinueToBackup = () => {
        navigate('/backup');
    };

    const handleBackToDiscover = () => {
        navigate('/discover');
    };

    // Auto-create plan if device is selected and no plan exists
    useEffect(() => {
        if (state.selectedDevice && !state.wipePlan && !state.isLoading) {
            handleCreatePlan();
        }
    }, [state.selectedDevice]);

    if (!state.selectedDevice) {
        return (
            <div style={{ maxWidth: '800px', margin: '0 auto' }}>
                <div className="card text-center" style={{ padding: '3rem' }}>
                    <div style={{ fontSize: '4rem', marginBottom: '1rem', opacity: 0.3 }}>📋</div>
                    <h3 className="font-semibold mb-2">No Device Selected</h3>
                    <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                        Please discover and select a device before creating a wipe plan.
                    </p>
                    <button
                        className="btn btn-primary"
                        onClick={handleBackToDiscover}
                    >
                        🔍 Go to Device Discovery
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
            <div className="mb-6">
                <h2 className="font-semibold mb-4" style={{ fontSize: '1.5rem' }}>
                    Wipe Plan Analysis
                </h2>
                <p style={{ color: '#64748b', marginBottom: '2rem' }}>
                    Non-destructive analysis of wiping strategy for {state.selectedDevice.model}.
                    This preview shows what would happen during actual wipe operations.
                </p>

                <div className="card mb-6">
                    <h3 className="font-semibold mb-4">Selected Device</h3>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span className="font-medium">Model:</span>
                            <div>{state.selectedDevice.model}</div>
                        </div>
                        <div>
                            <span className="font-medium">Serial:</span>
                            <div>{state.selectedDevice.serial}</div>
                        </div>
                        <div>
                            <span className="font-medium">Capacity:</span>
                            <div>{(state.selectedDevice.capacity / (1024 ** 3)).toFixed(1)} GB</div>
                        </div>
                        <div>
                            <span className="font-medium">Risk Level:</span>
                            <div>
                                <span className={`risk-badge ${state.selectedDevice.risk_level.toLowerCase()}`}>
                                    {state.selectedDevice.risk_level}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                <button
                    className="btn btn-primary mb-6"
                    onClick={handleCreatePlan}
                    disabled={state.isLoading}
                >
                    {state.isLoading ? '🔄 Analyzing...' : '📋 Analyze Wipe Plan'}
                </button>
            </div>

            {/* Wipe Plan Results */}
            {state.wipePlan && (
                <div className="mb-6">
                    <div className="card">
                        <div className="flex justify-between items-center mb-4">
                            <h3 className="font-semibold">Wipe Plan Summary</h3>
                            <div className="flex gap-2">
                                <button
                                    className={`btn ${!showJsonView ? 'btn-primary' : 'btn-secondary'} text-xs`}
                                    onClick={() => setShowJsonView(false)}
                                >
                                    Human Readable
                                </button>
                                <button
                                    className={`btn ${showJsonView ? 'btn-primary' : 'btn-secondary'} text-xs`}
                                    onClick={() => setShowJsonView(true)}
                                >
                                    JSON View
                                </button>
                            </div>
                        </div>

                        {!showJsonView ? (
                            <div className="grid grid-cols-2 gap-6">
                                <div>
                                    <h4 className="font-semibold mb-3">Strategy</h4>
                                    <div className="space-y-2 text-sm">
                                        <div>
                                            <span className="font-medium">Policy:</span>
                                            <span className="ml-2 px-2 py-1 bg-blue-100 text-blue-800 rounded text-xs">
                                                {state.wipePlan.policy}
                                            </span>
                                        </div>
                                        <div>
                                            <span className="font-medium">Method:</span>
                                            <div className="mt-1">{state.wipePlan.main_method}</div>
                                        </div>
                                        <div>
                                            <span className="font-medium">HPA/DCO Clear:</span>
                                            <span className={`ml-2 ${state.wipePlan.hpa_dco_clear ? 'text-green-600' : 'text-gray-600'}`}>
                                                {state.wipePlan.hpa_dco_clear ? '✓ Enabled' : '✗ Not needed'}
                                            </span>
                                        </div>
                                    </div>
                                </div>

                                <div>
                                    <h4 className="font-semibold mb-3">Verification</h4>
                                    <div className="space-y-2 text-sm">
                                        <div>
                                            <span className="font-medium">Sample Points:</span>
                                            <div className="mt-1">{state.wipePlan.verification.samples} random locations</div>
                                        </div>
                                        <div>
                                            <span className="font-medium">Coverage:</span>
                                            <div className="mt-1">
                                                {((state.wipePlan.verification.samples / 1000) * 100).toFixed(1)}% statistical coverage
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <pre className="log-viewer" style={{ height: '300px', fontSize: '0.75rem' }}>
                                {JSON.stringify(state.wipePlan, null, 2)}
                            </pre>
                        )}

                        {/* Blocked Status */}
                        {state.wipePlan.blocked && (
                            <div className="alert alert-error mt-4">
                                <h4 className="font-semibold mb-2">🚫 Wipe Operation Blocked</h4>
                                <p className="text-sm">
                                    {state.wipePlan.block_reason || 'This device cannot be wiped in current mode.'}
                                </p>
                                <p className="text-xs mt-2" style={{ opacity: 0.8 }}>
                                    Boot from SecureWipe ISO to enable wiping of system disks and mounted volumes.
                                </p>
                            </div>
                        )}
                    </div>
                </div>
            )}

            {/* Operation Logs */}
            {(state.logs.length > 0 || logs.length > 0) && (
                <div className="mb-6">
                    <LogViewer
                        logs={logs}
                        title="Plan Analysis Logs"
                    />
                </div>
            )}

            {/* Action Buttons */}
            {state.wipePlan && (
                <div className="card">
                    <h3 className="font-semibold mb-4">Next Steps</h3>

                    <div className="flex gap-4">
                        <button
                            className="btn btn-primary"
                            onClick={handleContinueToBackup}
                        >
                            📦 Continue to Backup
                            <div style={{ fontSize: '0.75rem', opacity: 0.8, marginTop: '0.25rem' }}>
                                Backup device data before wiping (recommended)
                            </div>
                        </button>

                        <button
                            className="btn btn-secondary"
                            onClick={handleBackToDiscover}
                        >
                            🔍 Select Different Device
                            <div style={{ fontSize: '0.75rem', opacity: 0.7, marginTop: '0.25rem' }}>
                                Return to device discovery
                            </div>
                        </button>
                    </div>

                    <div className="alert alert-info mt-4">
                        <h4 className="font-semibold mb-2">🛡️ MVP Safety Notice</h4>
                        <p className="text-sm">
                            This is a non-destructive preview mode. No actual wiping will occur in this demonstration version.
                            Real wipe execution is disabled for safety during evaluation.
                        </p>
                    </div>
                </div>
            )}

            {/* Current Operation Status */}
            {state.currentOperation && (
                <div className="alert alert-info mt-6">
                    <div className="flex items-center gap-2">
                        <div style={{
                            width: '20px',
                            height: '20px',
                            border: '2px solid #3b82f6',
                            borderTopColor: 'transparent',
                            borderRadius: '50%',
                            animation: 'spin 1s linear infinite'
                        }}></div>
                        <span>{state.currentOperation}</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default WipePlan;
