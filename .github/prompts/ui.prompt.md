---
title: "SecureWipe UI — Tauri + React"
description: "Design a one-click, safe UX that drives the Backup → Wipe → Certificates flow with strong guard rails."
labels: ["tauri", "react", "frontend", "ux", "security"]
audience: ["copilot", "developer"]
---

## Context

You are generating **TypeScript/React** code for the Tauri desktop app in `/ui`.  
This UI orchestrates the **core CLI** (from `/core`) via Tauri IPC/process calls and presents a clear, safe UX for non-technical users.

**Authoritative specs to follow** (read from repo):

- Product: `/docs/PRD.md`
- Schemas: `/docs/schemas.md`
- Certificates JSON: `/certs/schemas/*.json`

## UX Requirements

### Screens

1. **Welcome**

   - SIH banner and short mission statement.
   - Primary CTA: **“Backup & Wipe”**; Secondary: **“Wipe Only”**.

2. **Device Discovery**

   - Render device **cards**: model, capacity, bus, **masked serial**, and **Risk Badge** (CRITICAL/HIGH/SAFE).
   - If CRITICAL, show blocking state with tooltip: “Boot from ISO to wipe system disks.”

3. **Backup**

   - Destination selector (USB/NAS path).
   - Show **AES-256 encryption is ON**; no toggle in MVP.
   - Progress bar, bytes copied, ETA; post-run integrity report (N=5 sampling).
   - If backup fails, allow retry or “Skip and Wipe” (with warning).

4. **Wipe**

   - Policy dropdown: CLEAR / **PURGE** (default) / DESTROY (info).
   - Two-step confirmation (checkbox + modal) before starting.
   - Live step logs (HPA/DCO clear, sanitize/overwrite, verify sampling).
   - Clear failure messages with recovery suggestions.

5. **Certificates**
   - Show **4 buttons**: Backup JSON, Backup PDF, Wipe JSON, Wipe PDF.
   - Render **QR preview** and show **verification instructions** (CLI command and portal URL).
   - Show artifact save location and “Open Folder”.

### Interaction & IPC

- Shell out to `/core` CLI subcommands and stream progress to UI.
- All operations must be cancelable; prompt user to confirm cancellations during destructive steps.
- Persist last used destination path and UI preferences locally.

### Accessibility/Style

- Large CTAs, readable status colors, clear error banners.
- Consistent spacing and headings; minimal, modern theme.

## Constraints

- No cloud backup UI in MVP.
- Must refuse CRITICAL disk wipe operations unless running in “ISO mode” (UI receives this flag from env/config).
- Default export a React component for the main window; organize routes/components cleanly.

---

### Example Prompt to Start

Create a Tauri + React home screen with two buttons (“Backup & Wipe”, “Wipe Only”) and a placeholder device list that consumes JSON from a mock `discover` CLI call. Add a RiskBadge component with variants CRITICAL/HIGH/SAFE.

# end of file
