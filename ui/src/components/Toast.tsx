import { useEffect } from 'react';
import { useApp } from '../contexts/AppContext';

function Toast() {
    const { state, dispatch } = useApp();

    const removeToast = (id: string) => {
        dispatch({ type: 'REMOVE_TOAST', payload: id });
    };

    // Auto-dismiss toasts after 4 seconds
    useEffect(() => {
        state.toasts.forEach((toast) => {
            const timer = setTimeout(() => {
                removeToast(toast.id);
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
                    className={`alert alert-${toast.type}`}
                    style={{
                        minWidth: '300px',
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center'
                    }}
                >
                    <span>{toast.message}</span>
                    <button
                        onClick={() => removeToast(toast.id)}
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
