# Tauri-SecureWipe Integration

This implementation provides a secure bridge between the React frontend and the SecureWipe CLI binary via Tauri.

## Features

### Security & Guard Rails
- **Whitelisted commands**: Only `discover`, `wipe` (planning), `backup`, and `cert` subcommands are allowed
- **Destructive action blocking**: Any args containing `apply`, `execute`, `force`, `danger`, etc. are rejected
- **Wipe planning only**: The `wipe` command requires `--format` flag and blocks execution flags
- **Runtime limits**: Commands timeout after predefined limits (backup: 20min, discover/plan: 2min)
- **Output limiting**: Lines > 64KB are truncated to prevent memory issues

### Real-time Communication
- **Event-driven logs**: Stdout/stderr streams are emitted as Tauri events with timestamps
- **Live log buffer**: Frontend maintains rolling buffer of last 2000 log entries
- **Process lifecycle**: Exit events include exit codes and timestamps
- **Cancellation support**: Running processes can be cancelled via session IDs

## Backend Implementation (Rust/Tauri)

### Commands
- `run_securewipe(args, session_id)` - Execute CLI with validation and streaming
- `cancel_securewipe(session_id)` - Kill running process by session ID

### Events Emitted
- `securewipe://stdout` - { line: string, ts: string, stream: "stdout" }
- `securewipe://stderr` - { line: string, ts: string, stream: "stderr" }  
- `securewipe://exit` - { code: number|null, ts: string }

## Frontend Implementation (React/TypeScript)

### Hook: `useSecureWipe()`
```typescript
const {
  logs,           // LogEvent[] - real-time log entries
  running,        // boolean - is command executing
  discover,       // () => Promise<Device[]>
  planWipe,       // (opts) => Promise<WipePlan>
  backup,         // (opts) => Promise<BackupResult>
  cancel,         // () => Promise<void>
  clearLogs       // () => void
} = useSecureWipe();
```

### High-level Methods
- `discover(options?)` - Get device list with risk levels
- `planWipe({ device, samples?, isoMode?, noEnrich? })` - Create wipe strategy (safe)
- `backup({ device, dest, sign?, signKeyPath?, includePaths? })` - Run encrypted backup

## Usage Examples

### Device Discovery
```typescript
const devices = await discover();
// Returns Device[] with risk_level: 'SAFE' | 'HIGH' | 'CRITICAL'
```

### Wipe Planning (Non-destructive)
```typescript
const plan = await planWipe({ 
  device: '/dev/sdb', 
  samples: 128,
  isoMode: false 
});
// Returns WipePlan with policy, method, verification details
```

### Backup with Signing
```typescript
const result = await backup({
  device: '/dev/sdb',
  dest: '~/SecureWipe/backups',
  sign: true,
  signKeyPath: '~/keys/private.pem',
  includePaths: ['/home/user/Documents']
});
// Returns paths to generated certificates and manifests
```

## Testing

Use the `SecureWipeTest` component to verify integration:

1. Start Tauri dev server: `npm run tauri dev`
2. Add `<SecureWipeTest />` to your app
3. Test each operation and verify logs stream correctly
4. Ensure destructive operations are blocked

## CLI Requirements

The frontend expects `securewipe` binary in PATH with these subcommands:
- `securewipe discover --format json [--no-enrich]`
- `securewipe wipe --device <path> --format json --samples <n> [--iso-mode] [--no-enrich]`
- `securewipe backup --device <path> --dest <path> [--sign] [--sign-key-path <path>] [--paths <paths>]`
- `securewipe cert sign --file <path> --sign-key-path <path>`
- `securewipe cert verify --file <path>`

## Error Handling

- **CLI not found**: Clear error message prompting user to install/configure SecureWipe
- **Invalid args**: Validation errors are returned immediately
- **Process timeout**: Processes are killed and reported as failed
- **Parse errors**: Robust JSON parsing with fallbacks for malformed output

## Security Notes

- Private keys are never embedded in the frontend/backend code
- Destructive wipe execution is completely blocked in MVP
- All process execution is sandboxed via Tauri's security model
- Log output is sanitized and size-limited