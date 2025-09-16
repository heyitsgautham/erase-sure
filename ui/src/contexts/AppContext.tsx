import { createContext, useContext, useReducer, ReactNode } from 'react';

// Types
export interface Device {
    path: string;
    model: string;
    serial: string;
    capacity: number;
    bus: string;
    mountpoints: string[];
    risk_level: 'CRITICAL' | 'HIGH' | 'SAFE';
    blocked: boolean;
    block_reason?: string;
}

export interface WipePlan {
    device_path: string;
    policy: 'CLEAR' | 'PURGE' | 'DESTROY';
    main_method: string;
    hpa_dco_clear: boolean;
    verification: {
        samples: number;
    };
    blocked: boolean;
    block_reason?: string;
}

export interface BackupResult {
    backup_path: string;
    certificate_json_path?: string;
    certificate_pdf_path?: string;
    manifest_path: string;
    integrity_checks: number;
}

export interface ToastMessage {
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
    duration?: number;
}

interface ProgressInfo {
    title: string;
    currentStep: number;
    totalSteps: number;
    currentStepName: string;
    percentage?: number;
}

interface AppState {
    devices: Device[];
    selectedDevice: Device | null;
    wipePlan: WipePlan | null;
    backupResult: BackupResult | null;
    isLoading: boolean;
    currentOperation: string | null;
    progress: ProgressInfo | null;
    logs: string[];
    toasts: ToastMessage[];
}

type AppAction =
    | { type: 'SET_DEVICES'; payload: Device[] }
    | { type: 'SELECT_DEVICE'; payload: Device | null }
    | { type: 'SET_WIPE_PLAN'; payload: WipePlan | null }
    | { type: 'SET_BACKUP_RESULT'; payload: BackupResult | null }
    | { type: 'SET_LOADING'; payload: boolean }
    | { type: 'SET_OPERATION'; payload: string | null }
    | { type: 'SET_PROGRESS'; payload: ProgressInfo | null }
    | { type: 'ADD_LOG'; payload: string }
    | { type: 'CLEAR_LOGS' }
    | { type: 'ADD_TOAST'; payload: ToastMessage }
    | { type: 'REMOVE_TOAST'; payload: string };

const initialState: AppState = {
    devices: [],
    selectedDevice: null,
    wipePlan: null,
    backupResult: null,
    isLoading: false,
    currentOperation: null,
    progress: null,
    logs: [],
    toasts: []
};

function appReducer(state: AppState, action: AppAction): AppState {
    switch (action.type) {
        case 'SET_DEVICES':
            return { ...state, devices: action.payload };
        case 'SELECT_DEVICE':
            return { ...state, selectedDevice: action.payload };
        case 'SET_WIPE_PLAN':
            return { ...state, wipePlan: action.payload };
        case 'SET_BACKUP_RESULT':
            return { ...state, backupResult: action.payload };
        case 'SET_LOADING':
            return { ...state, isLoading: action.payload };
        case 'SET_OPERATION':
            return { ...state, currentOperation: action.payload };
        case 'SET_PROGRESS':
            return { ...state, progress: action.payload };
        case 'ADD_LOG':
            return { ...state, logs: [...state.logs, action.payload] };
        case 'CLEAR_LOGS':
            return { ...state, logs: [] };
        case 'ADD_TOAST':
            return { ...state, toasts: [...state.toasts, action.payload] };
        case 'REMOVE_TOAST':
            return { ...state, toasts: state.toasts.filter(t => t.id !== action.payload) };
        default:
            return state;
    }
}

interface AppContextType {
    state: AppState;
    dispatch: React.Dispatch<AppAction>;
    // Helper functions
    addToast: (message: string, type?: ToastMessage['type'], duration?: number) => void;
    clearLogs: () => void;
    addLog: (log: string) => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
    const [state, dispatch] = useReducer(appReducer, initialState);

    const addToast = (message: string, type: ToastMessage['type'] = 'info', duration = 5000) => {
        const id = Math.random().toString(36).substr(2, 9);
        dispatch({ type: 'ADD_TOAST', payload: { id, message, type, duration } });

        if (duration > 0) {
            setTimeout(() => {
                dispatch({ type: 'REMOVE_TOAST', payload: id });
            }, duration);
        }
    };

    const clearLogs = () => {
        dispatch({ type: 'CLEAR_LOGS' });
    };

    const addLog = (log: string) => {
        dispatch({ type: 'ADD_LOG', payload: log });
    };

    return (
        <AppContext.Provider value={{ state, dispatch, addToast, clearLogs, addLog }}>
            {children}
        </AppContext.Provider>
    );
}

export function useApp() {
    const context = useContext(AppContext);
    if (context === undefined) {
        throw new Error('useApp must be used within an AppProvider');
    }
    return context;
}
