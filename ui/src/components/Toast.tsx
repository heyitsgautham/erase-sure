import { useEffect, useState } from 'react';
import { useApp } from '../contexts/AppContext';

function Toast() {
    const { state, dispatch } = useApp();
    const [fadingToasts, setFadingToasts] = useState<Set<string>>(new Set());

    const removeToast = (id: string) => {
        dispatch({ type: 'REMOVE_TOAST', payload: id });
    };

    const startFadeOut = (id: string) => {
        setFadingToasts(prev => new Set([...prev, id]));
        setTimeout(() => {
            removeToast(id);
            setFadingToasts(prev => {
                const newSet = new Set(prev);
                newSet.delete(id);
                return newSet;
            });
        }, 300); // Fade duration
    };

    // Auto-dismiss toasts after 4 seconds
    useEffect(() => {
        state.toasts.forEach((toast) => {
            const timer = setTimeout(() => {
                startFadeOut(toast.id);
            }, 4000); // 4 seconds

            // Clear timeout if component unmounts or toast is manually removed
            return () => clearTimeout(timer);
        });
    }, [state.toasts]);

    if (state.toasts.length === 0) {
        return null;
    }

    return (
        <div style={{
            position: 'fixed',
            top: '1rem',
            right: '1rem',
            zIndex: 1000,
            display: 'flex',
            flexDirection: 'column',
            gap: '0.5rem'
        }}>
            {state.toasts.map((toast) => (
                <div
                    key={toast.id}
                    className={`alert alert-${toast.type} toast-item ${fadingToasts.has(toast.id) ? 'toast-fade-out' : ''}`}
                    style={{
                        minWidth: '300px',
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        transition: 'opacity 0.3s ease, transform 0.3s ease'
                    }}
                >
                    <span>{toast.message}</span>
                    <button
                        onClick={() => startFadeOut(toast.id)}
                        style={{
                            background: 'none',
                            border: 'none',
                            fontSize: '1.2rem',
                            cursor: 'pointer',
                            marginLeft: '1rem'
                        }}
                    >
                        ×
                    </button>
                </div>
            ))}
        </div>
    );
}

export default Toast;
