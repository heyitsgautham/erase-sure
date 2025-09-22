# SecureWipe UI

Modern React + Tauri desktop application for the SecureWipe MVP, providing a user-friendly interface for secure disk wiping operations with backup and verification capabilities.

## Features

### 🏠 Home Screen
- SIH 2024 branding and mission statement
- Two primary workflows: "Backup & Wipe" and "Wipe Plan Only"
- Feature highlights and MVP safety notice
- Modern gradient design with clear CTAs

### 🔍 Device Discovery
- Automatic device scanning with mock CLI integration
- Device cards showing model, capacity, serial, and risk levels
- Risk badges: CRITICAL (red), HIGH (amber), SAFE (green)
- Interactive device selection with blocking for system disks
- Real-time device statistics and warnings

### 📋 Wipe Plan Analysis
- Non-destructive wipe strategy preview
- Human-readable and JSON view toggle
- Policy selection (CLEAR/PURGE, with DESTROY info)
- HPA/DCO detection and sampling configuration
- Blocking notifications for critical devices
- Live operation logs

### 📦 Encrypted Backup
- AES-256-CTR encryption with PBKDF2 key derivation
- Destination folder selection with file browser
- Custom include paths for selective backup
- Optional signing key configuration
- Real-time progress monitoring with integrity checks
- Certificate generation (JSON + PDF)

### 📜 Certificate Management
- Certificate browser with filtering and search
- QR code generation for mobile verification
- One-click portal verification links
- File management (JSON/PDF access)
- Verification instructions (CLI + Web)
- Certificate timeline and metadata

## Architecture

### Component Structure
```
src/
├── components/
│   ├── Navigation.tsx      # Top navigation bar
│   ├── Toast.tsx          # Notification system
│   ├── DeviceCard.tsx     # Device display with risk badges
│   ├── LogViewer.tsx      # Real-time log streaming
│   ├── QRPreview.tsx      # QR code generation
│   └── FileLink.tsx       # File/folder opening utility
├── screens/
│   ├── Home.tsx           # Landing page with CTAs
│   ├── Discover.tsx       # Device discovery and selection
│   ├── WipePlan.tsx       # Non-destructive wipe planning
│   ├── Backup.tsx         # Encrypted backup workflow
│   └── Certificates.tsx   # Certificate management
├── contexts/
│   └── AppContext.tsx     # Global state management
├── hooks/
│   └── useSecureWipe.ts   # CLI integration layer
└── styles.css             # Global styles and utilities
```

### State Management
- React Context for global app state
- Device selection persistence
- Operation progress tracking
- Log streaming and toast notifications
- Certificate catalog management

### CLI Integration
- Mock implementation for MVP demonstration
- Structured JSON output parsing
- Real-time log streaming simulation
- Error handling and recovery
- Process lifecycle management

## Technology Stack

- **Framework**: React 18 + TypeScript
- **Desktop**: Tauri (Rust backend)
- **Router**: React Router DOM
- **Bundler**: Vite
- **Styling**: Custom CSS with utility classes
- **Icons**: Emoji + Lucide React (optional)
- **QR Codes**: qrcode library

## Getting Started

### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Tauri CLI

### Installation
```bash
# Install dependencies
npm install

# Install Tauri CLI
npm install -g @tauri-apps/cli

# Development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Mock Data
The UI includes comprehensive mock data for demonstration:
- 3 sample devices (Samsung SSD, WD NVMe, SanDisk USB)
- Risk level simulation (CRITICAL system disk, SAFE external)
- Realistic backup and wipe operation logs
- Sample certificates with verification URLs

## MVP Safety Features

### Non-Destructive Mode
- All wipe operations are simulated
- No actual disk modification occurs
- Clear safety notices throughout UI
- Blocking for critical system devices
- Educational tooltips and warnings

### Demonstration Flow
1. **Discovery**: Shows mixed device types with risk assessment
2. **Planning**: Displays realistic wipe strategies without execution
3. **Backup**: Simulates encrypted backup with progress and certificates
4. **Verification**: Generates QR codes and portal integration links

## Integration Points

### CLI Bridge
```typescript
// Mock implementation in useSecureWipe.ts
const { discoverDevices, createWipePlan, runBackup } = useSecureWipe();

// Real implementation would use:
await Command.create('securewipe', ['discover', '--format', 'json']).execute();
```

### Portal Integration
- QR codes link to verification portal
- Direct browser launch for certificate checking
- Mobile-friendly verification workflow
- Offline CLI verification instructions

### File System
- Certificate storage in `~/SecureWipe/certificates/`
- Backup destination configuration
- Cross-platform path handling
- File browser integration via Tauri

## Customization

### Theming
Modify `src/styles.css` for custom branding:
```css
:root {
  --primary-color: #3b82f6;
  --success-color: #16a34a;
  --warning-color: #d97706;
  --error-color: #dc2626;
}
```

### Mock Data
Update mock responses in `useSecureWipe.ts`:
- Device configurations
- Operation timing
- Certificate generation
- Error scenarios

## Development Notes

### TypeScript Errors
Current lint errors are expected due to missing React types in development environment. These will resolve when dependencies are installed:
```bash
npm install  # Installs React types and dependencies
```

### Production Deployment
1. Replace mock CLI integration with real Tauri commands
2. Implement file system APIs for certificate management
3. Add real QR code generation with `qrcode` library
4. Configure Tauri security policies and permissions
5. Add proper error handling and recovery mechanisms

## Security Considerations

- No sensitive data in localStorage
- Secure file path handling via Tauri APIs
- Certificate validation before display
- Process isolation for CLI operations
- Memory cleanup for large operations

## Contributing

1. Follow TypeScript strict mode guidelines
2. Maintain component isolation and reusability
3. Add comprehensive error handling
4. Include accessibility features (ARIA labels, keyboard nav)
5. Test across different screen sizes and device types

## License

Part of the SecureWipe project for Smart India Hackathon 2024.
