# erase-sure

## Project Structure

```
secure-wipe-mvp/
├── package.json                 # Dependencies and scripts
├── main.js                     # Electron main process
├── src/
│   ├── core/                   # Shared business logic
│   │   ├── PolicyEngine.js     # NIST SP 800-88 implementation
│   │   ├── MediaDiscovery.js   # Device detection and classification
│   │   ├── WipeOrchestrator.js # Wipe execution coordinator
│   │   ├── CertificateGenerator.js # JSON + PDF certificate creation
│   │   └── EventBus.js         # Inter-module communication
│   ├── backends/
│   │   ├── LinuxBackend.js     # Linux-specific wipe implementations
│   │   └── ShellExecutor.js    # Safe shell command execution
│   ├── ui/
│   │   ├── index.html          # Main application UI
│   │   ├── renderer.js         # UI logic (Electron renderer)
│   │   └── styles.css          # Application styling
│   └── verification/
│       ├── server.js           # Certificate verification web server
│       └── validator.js        # Certificate validation logic
├── certificates/              # Generated certificates storage
├── logs/                      # Wipe execution logs
└── keys/                      # Private/public keys for signing
    ├── private.pem
    └── public.pem
```
