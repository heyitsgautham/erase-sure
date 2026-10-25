import { createContext, useCallback, useContext, useMemo, useReducer, ReactNode } from 'react';
import type { LogEvent } from '../types/securewipe';

/** The log keeps the newest lines only, so a long wipe cannot grow it without bound. */
export const MAX_LOG_LINES = 2000;

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
    policy: 'CLEAR' | 'PURGE';
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

export interface ProgressInfo {
    title: string;
    currentStep: number;
    totalSteps: number;
    currentStepName: string;
    percentage?: number;
}

export interface AppState {
    devices: Device[];
    selectedDevice: Device | null;
    wipePlan: WipePlan | null;
    backupResult: BackupResult | null;
    currentOperation: string | null;
    progress: ProgressInfo | null;
    logs: LogEvent[];
    toasts: ToastMessage[];
    operationType: 'backup' | 'wipe' | null; // Track intended operation type
}

type AppAction =
    | { type: 'SET_DEVICES'; payload: Device[] }
    | { type: 'SELECT_DEVICE'; payload: Device | null }
    | { type: 'SET_WIPE_PLAN'; payload: WipePlan | null }
    | { type: 'SET_BACKUP_RESULT'; payload: BackupResult | null }
    | { type: 'SET_OPERATION'; payload: string | null }
    | { type: 'SET_PROGRESS'; payload: ProgressInfo | null }
    | { type: 'ADD_LOG'; payload: LogEvent }
    | { type: 'CLEAR_LOGS' }
    | { type: 'ADD_TOAST'; payload: ToastMessage }
    | { type: 'REMOVE_TOAST'; payload: string }
    | { type: 'SET_OPERATION_TYPE'; payload: 'backup' | 'wipe' | null };

const initialState: AppState = {
    devices: [],
    selectedDevice: null,
    wipePlan: null,
    backupResult: null,
    currentOperation: null,
    progress: null,
    logs: [],
    toasts: [],
    operationType: null
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
        case 'SET_OPERATION':
            return { ...state, currentOperation: action.payload };
        case 'SET_PROGRESS':
            return { ...state, progress: action.payload };
        case 'ADD_LOG':
            return { ...state, logs: [...state.logs, action.payload].slice(-MAX_LOG_LINES) };
        case 'CLEAR_LOGS':
            return { ...state, logs: [] };
        case 'ADD_TOAST':
            return { ...state, toasts: [...state.toasts, action.payload] };
        case 'REMOVE_TOAST':
            return { ...state, toasts: state.toasts.filter(t => t.id !== action.payload) };
        case 'SET_OPERATION_TYPE':
            return { ...state, operationType: action.payload };
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
    addLog: (log: LogEvent) => void;
    clearDeviceSelection: () => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children, initial }: { children: ReactNode; initial?: Partial<AppState> }) {
    const [state, dispatch] = useReducer(appReducer, { ...initialState, ...initial });

    // The helpers keep one identity for the provider's lifetime. Screens list them as effect
    // dependencies, so a new function per render would re-run those effects on every state change.
    const addToast = useCallback((message: string, type: ToastMessage['type'] = 'info', duration = 5000) => {
        const id = Math.random().toString(36).slice(2, 11);
        dispatch({ type: 'ADD_TOAST', payload: { id, message, type, duration } });

        if (duration > 0) {
            setTimeout(() => {
                dispatch({ type: 'REMOVE_TOAST', payload: id });
            }, duration);
        }
    }, []);

    const clearLogs = useCallback(() => {
        dispatch({ type: 'CLEAR_LOGS' });
    }, []);

    const addLog = useCallback((log: LogEvent) => {
        dispatch({ type: 'ADD_LOG', payload: log });
    }, []);

    const clearDeviceSelection = useCallback(() => {
        dispatch({ type: 'SELECT_DEVICE', payload: null });
    }, []);

    const value = useMemo(
        () => ({ state, dispatch, addToast, clearLogs, addLog, clearDeviceSelection }),
        [state, addToast, clearLogs, addLog, clearDeviceSelection],
    );

    return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

// The hook lives beside its provider on purpose; fast refresh does a full reload for this file.
// eslint-disable-next-line react-refresh/only-export-components
export function useApp() {
    const context = useContext(AppContext);
    if (context === undefined) {
        throw new Error('useApp must be used within an AppProvider');
    }
    return context;
}
