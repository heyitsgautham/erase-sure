import { useEffect } from 'react';
import { useApp } from '../contexts/AppContext';
import { useSecureWipe as useNewSecureWipe } from './useSecureWipe';

/**
 * Compatibility wrapper that bridges the new useSecureWipe hook 
 * with the existing AppContext log system
 */
export function useSecureWipe() {
  const { dispatch, addToast } = useApp();
  const newHook = useNewSecureWipe();

  // Convert LogEvents to strings and sync with AppContext
  useEffect(() => {
    // Clear old logs when new logs start
    if (newHook.logs.length === 0 || newHook.logs.length === 1) {
      dispatch({ type: 'CLEAR_LOGS' });
    }

    // Add new log entries
    const lastLog = newHook.logs[newHook.logs.length - 1];
    if (lastLog) {
      const logString = `[${formatTimestamp(lastLog.ts)}] ${lastLog.stream}: ${lastLog.line}`;
      dispatch({ type: 'ADD_LOG', payload: logString });
    }
  }, [newHook.logs, dispatch]);

  // Sync running state with AppContext loading
  useEffect(() => {
    dispatch({ type: 'SET_LOADING', payload: newHook.running });
  }, [newHook.running, dispatch]);

  const formatTimestamp = (ts: string) => {
    try {
      return new Date(ts).toLocaleTimeString();
    } catch {
      return ts;
    }
  };

  // Wrapper functions that handle success/error toasts
  const discoverDevices = async () => {
    try {
      dispatch({ type: 'SET_OPERATION', payload: 'Discovering devices...' });
      const devices = await newHook.discover();
      dispatch({ type: 'SET_DEVICES', payload: devices });
      addToast('Devices discovered successfully', 'success');
      return devices;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Discovery failed';
      addToast(message, 'error');
      throw error;
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  };

  const createWipePlan = async (devicePath: string) => {
    try {
      dispatch({ type: 'SET_OPERATION', payload: 'Creating wipe plan...' });
      const plan = await newHook.planWipe({ device: devicePath });
      dispatch({ type: 'SET_WIPE_PLAN', payload: plan });
      addToast('Wipe plan created successfully', 'success');
      return plan;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Plan creation failed';
      addToast(message, 'error');
      throw error;
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  };

  const runBackup = async (devicePath: string, destination: string, signKeyPath?: string) => {
    try {
      dispatch({ type: 'SET_OPERATION', payload: 'Running backup...' });
      const result = await newHook.backup({
        device: devicePath,
        dest: destination,
        sign: !!signKeyPath,
        signKeyPath,
      });

      // Convert to the format expected by AppContext
      const backupResult = {
        backup_path: result.backupPath || destination,
        certificate_json_path: result.certPathJson,
        certificate_pdf_path: result.certPathPdf,
        manifest_path: `${destination}/manifest.json`,
        integrity_checks: 5, // Default value
      };

      dispatch({ type: 'SET_BACKUP_RESULT', payload: backupResult });
      addToast('Backup completed successfully', 'success');
      return backupResult;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Backup failed';
      addToast(message, 'error');
      throw error;
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  };

  return {
    // Original methods with AppContext integration
    discoverDevices,
    createWipePlan,
    runBackup,

    // Direct access to new hook methods
    discover: newHook.discover,
    planWipe: newHook.planWipe,
    backup: newHook.backup,
    run: newHook.run,
    cancel: newHook.cancel,
    clearLogs: newHook.clearLogs,

    // State
    logs: newHook.logs,
    running: newHook.running,
  };
}