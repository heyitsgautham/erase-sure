---
applyTo:
  - "ui/**"
---

# Copilot Instructions for UI (Tauri + React)

- Use **TypeScript/React** with Tauri for all code under /ui.
- UI must orchestrate `/core` CLI commands through Tauri IPC.
- Device list must always display **Risk Badges** (CRITICAL/HIGH/SAFE).
- Backup UI:
  - Default to AES-256 encryption, always ON.
  - Destination limited to USB/NAS for MVP.
- Wipe UI:
  - Default wipe policy = PURGE.
  - Require two-step confirmation before destructive actions.
- Certificates UI:
  - Must display buttons for Backup JSON/PDF and Wipe JSON/PDF.
  - Must render QR code preview and show verification instructions.
- Out of scope for MVP:
  - Cloud backup UI
  - Multi-language support
