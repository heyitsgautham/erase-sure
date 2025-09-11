import { useCallback } from 'react';
import { useApp } from '../contexts/AppContext';
import type { Device, WipePlan, BackupResult } from '../contexts/AppContext';

interface ProcessResult {
  success: boolean;
  stdout: string;
  stderr: string;
  code: number;
}

export function useSecureWipe() {
  const { dispatch, addToast, addLog } = useApp();

  const runCommand = useCallback(async (
    command: string,
    args: string[] = [],
    streamLogs = false
  ): Promise<ProcessResult> => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      if (streamLogs) {
        dispatch({ type: 'CLEAR_LOGS' });
      }

      // For now, we'll use a mock implementation
      // In production, this would use Tauri's Command API
      const result = await mockCliCommand(command, args, streamLogs ? addLog : undefined);
      
      if (result.success) {
        addToast('Command completed successfully', 'success');
      } else {
        addToast(`Command failed: ${result.stderr}`, 'error');
      }

      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      addToast(`Error: ${errorMessage}`, 'error');
      return {
        success: false,
        stdout: '',
        stderr: errorMessage,
        code: 1
      };
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, [dispatch, addToast, addLog]);

  const discoverDevices = useCallback(async (): Promise<Device[]> => {
    dispatch({ type: 'SET_OPERATION', payload: 'Discovering devices...' });
    
    try {
      const result = await runCommand('securewipe', ['discover', '--format', 'json']);
      
      if (result.success) {
        const devices: Device[] = JSON.parse(result.stdout);
        dispatch({ type: 'SET_DEVICES', payload: devices });
        return devices;
      } else {
        throw new Error(result.stderr || 'Failed to discover devices');
      }
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  }, [runCommand, dispatch]);

  const createWipePlan = useCallback(async (devicePath: string): Promise<WipePlan> => {
    dispatch({ type: 'SET_OPERATION', payload: 'Creating wipe plan...' });
    
    try {
      const result = await runCommand('securewipe', [
        'wipe',
        '--device', devicePath,
        '--format', 'json',
        '--samples', '128',
        '--plan-only'  // This flag would prevent actual wiping
      ], true);
      
      if (result.success) {
        const wipePlan: WipePlan = JSON.parse(result.stdout);
        dispatch({ type: 'SET_WIPE_PLAN', payload: wipePlan });
        return wipePlan;
      } else {
        throw new Error(result.stderr || 'Failed to create wipe plan');
      }
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  }, [runCommand, dispatch]);

  const runBackup = useCallback(async (
    devicePath: string,
    destination: string,
    signKeyPath?: string
  ): Promise<BackupResult> => {
    dispatch({ type: 'SET_OPERATION', payload: 'Running backup...' });
    
    try {
      const args = [
        'backup',
        '--device', devicePath,
        '--dest', destination,
        '--sign'
      ];
      
      if (signKeyPath) {
        args.push('--sign-key-path', signKeyPath);
      }

      const result = await runCommand('securewipe', args, true);
      
      if (result.success) {
        // Parse the backup result from logs/stdout
        const backupResult: BackupResult = parseBackupResult(result.stdout);
        dispatch({ type: 'SET_BACKUP_RESULT', payload: backupResult });
        return backupResult;
      } else {
        throw new Error(result.stderr || 'Backup failed');
      }
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  }, [runCommand, dispatch]);

  const signCertificate = useCallback(async (
    certPath: string,
    signKeyPath: string
  ): Promise<string> => {
    dispatch({ type: 'SET_OPERATION', payload: 'Signing certificate...' });
    
    try {
      const result = await runCommand('securewipe', [
        'cert',
        'sign',
        '--file', certPath,
        '--sign-key-path', signKeyPath
      ]);
      
      if (result.success) {
        return result.stdout.trim();
      } else {
        throw new Error(result.stderr || 'Failed to sign certificate');
      }
    } finally {
      dispatch({ type: 'SET_OPERATION', payload: null });
    }
  }, [runCommand, dispatch]);

  return {
    discoverDevices,
    createWipePlan,
    runBackup,
    signCertificate,
    runCommand
  };
}

// Mock implementation for development
async function mockCliCommand(
  command: string,
  args: string[],
  onLog?: (log: string) => void
): Promise<ProcessResult> {
  const fullCommand = `${command} ${args.join(' ')}`;
  
  if (onLog) {
    onLog(`Running: ${fullCommand}`);
  }

  // Simulate command execution delay
  await new Promise(resolve => setTimeout(resolve, 1000));

  if (args.includes('discover')) {
    const mockDevices: Device[] = [
      {
        path: '/dev/sdb',
        model: 'Samsung SSD 980 PRO',
        serial: 'S5P2********1234',
        capacity: 1000204886016,
        bus: 'nvme',
        mountpoints: [],
        risk_level: 'SAFE',
        blocked: false
      },
      {
        path: '/dev/sdc',
        model: 'WD Blue SN570',
        serial: 'WD-WX********5678',
        capacity: 500107862016,
        bus: 'nvme',
        mountpoints: ['/'],
        risk_level: 'CRITICAL',
        blocked: true,
        block_reason: 'System disk with active mount points'
      },
      {
        path: '/dev/sdd',
        model: 'SanDisk Ultra USB',
        serial: 'SD********9012',
        capacity: 32017047552,
        bus: 'usb',
        mountpoints: [],
        risk_level: 'HIGH',
        blocked: false
      }
    ];

    if (onLog) {
      onLog('Scanning storage devices...');
      onLog(`Found ${mockDevices.length} devices`);
    }

    return {
      success: true,
      stdout: JSON.stringify(mockDevices, null, 2),
      stderr: '',
      code: 0
    };
  }

  if (args.includes('wipe') && args.includes('--plan-only')) {
    const mockPlan: WipePlan = {
      device_path: args[args.indexOf('--device') + 1],
      policy: 'PURGE',
      main_method: 'ATA Secure Erase Enhanced',
      hpa_dco_clear: true,
      verification: {
        samples: 128
      },
      blocked: false
    };

    if (onLog) {
      onLog('Analyzing device capabilities...');
      onLog('Checking for HPA/DCO areas...');
      onLog('Creating wipe plan...');
      onLog(`Plan created: ${mockPlan.main_method}`);
    }

    return {
      success: true,
      stdout: JSON.stringify(mockPlan, null, 2),
      stderr: '',
      code: 0
    };
  }

  if (args.includes('backup')) {
    if (onLog) {
      onLog('Starting encrypted backup...');
      onLog('Creating manifest...');
      onLog('Copying files: 1.2GB / 4.8GB (25%)');
      onLog('Copying files: 2.4GB / 4.8GB (50%)');
      onLog('Copying files: 3.6GB / 4.8GB (75%)');
      onLog('Copying files: 4.8GB / 4.8GB (100%)');
      onLog('Performing integrity checks...');
      onLog('Generating certificates...');
      onLog('Backup completed successfully');
    }

    return {
      success: true,
      stdout: 'Backup completed successfully\nCertificate JSON: ~/SecureWipe/certificates/backup_cert_20250910_143022.json\nCertificate PDF: ~/SecureWipe/certificates/backup_cert_20250910_143022.pdf',
      stderr: '',
      code: 0
    };
  }

  // Default response for unrecognized commands
  if (onLog) {
    onLog(`Command executed: ${fullCommand}`);
  }

  return {
    success: true,
    stdout: 'Command completed',
    stderr: '',
    code: 0
  };
}

function parseBackupResult(stdout: string): BackupResult {
  // Parse the output to extract file paths
  const lines = stdout.split('\n');
  const result: BackupResult = {
    backup_path: '~/SecureWipe/backups/backup_20250910_143022',
    manifest_path: '~/SecureWipe/backups/backup_20250910_143022/manifest.json',
    integrity_checks: 5
  };

  for (const line of lines) {
    if (line.includes('Certificate JSON:')) {
      result.certificate_json_path = line.split('Certificate JSON: ')[1]?.trim();
    } else if (line.includes('Certificate PDF:')) {
      result.certificate_pdf_path = line.split('Certificate PDF: ')[1]?.trim();
    }
  }

  return result;
}
