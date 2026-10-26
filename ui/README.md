# SecureWipe desktop app

The desktop app is a Tauri 1 window around a React 18 and TypeScript UI.
It runs the `securewipe` CLI from `core/` for every disk operation and streams the CLI's output into guided screens for discovery, backup, wipe planning, destructive wiping and certificates.
The app never touches a disk itself.

## Run it

```bash
cd ui
npm ci
npm run tauri dev        # the window, with the Rust backend and the CLI
npm run dev              # the React app alone on http://localhost:1420
```

`npm run tauri dev` looks for the CLI at `core/target/release/securewipe`, then `core/target/debug/securewipe`, then `securewipe` on `PATH`.
Build it first with `cd core && cargo build`.

`npm run dev` has no backend.
Screens render, and any action that needs the disk shows "The SecureWipe desktop backend is not running".

## Configuration

| Variable | Default | Read by |
|---|---|---|
| `VITE_PORTAL_URL` | `http://localhost:8000` | `src/lib/config.ts`, for "Verify Online", certificate QR links and "Open in Portal Verify" |

Copy `.env.example` to `.env` in this folder to change it.
Vite builds the value into the app, so rebuild after a change.
Only `http` and `https` URLs are accepted.

## Checks

| Command | What it does |
|---|---|
| `npm run lint` | ESLint 9 with `eslint.config.js`, zero warnings allowed, bare `console` calls rejected |
| `npm run format:check` | Prettier with `.prettierrc.json` |
| `npm run typecheck` | `tsc --noEmit` in strict mode |
| `npm run test:coverage` | Vitest on jsdom; fails under 90% of lines or 85% of branches |
| `npm run build` | type check, then the Vite production bundle in `dist/` |
| `cd src-tauri && cargo test` | the Rust backend's integration tests; needs WebKitGTK 4.0 headers on Linux and a built `dist/` |

CI runs every one of these in the `UI (React + Vite)` and `Desktop backend (Tauri)` jobs.

## How it fits together

```
screen (src/screens)
  └─ hook (src/hooks)                 useSecureWipe, useBackupRun, useCertificates, useFileBrowser
       └─ commands (src/lib/commands.ts)   one typed wrapper per Tauri command, result checked by a zod schema
            └─ Tauri command (src-tauri/src/main.rs)
                 └─ library (src-tauri/src/{args,cli,files,wipe}.rs)
                      └─ securewipe CLI process
                           └─ stdout, stderr and exit arrive as securewipe:// events
                                └─ useCliLogStream ─► app state logs ─► LogViewer and progress
```

A screen asks a hook for an operation.
The hook calls a typed command, the Rust side checks the arguments and spawns the CLI, and the CLI's lines come back as events.
`useCliLogStream`, mounted once in `App`, is the only subscriber that copies lines into the app log, so each line appears once.
`run()` in `useSecureWipe` subscribes to the exit event before it spawns the CLI, so a fast CLI cannot finish unseen.

### Source layout

| Folder | Holds |
|---|---|
| `src/screens/` | one component per route; `Backup`, `Certificates` and the four scan screens are thin and delegate to the folders below |
| `src/components/` | shared pieces such as `DeviceCard`, `LogViewer`, `Progress`, `FileBrowser` and `ErrorBoundary` |
| `src/components/backup/`, `certificates/`, `discovery/` | the parts of the larger screens |
| `src/hooks/` | the operations: running the CLI, a backup run with its progress bar, the certificate list, the file browser |
| `src/lib/` | logic with no React in it: CLI output parsing, backup arguments and outcomes, progress steps, schemas, typed commands, the logger, routes and config |
| `src/contexts/AppContext.tsx` | app state: devices, selection, plan, backup result, progress, logs and toasts |
| `src/types/securewipe.ts` | the event and result shapes that cross the Tauri boundary |
| `src/test/` | the fake Tauri backend and render helpers the tests use |
| `src-tauri/` | the Rust side of the app |

### Logging

`src/lib/logger.ts` writes one JSON object per entry, with `ts`, `level`, `module`, `msg`, optional `data` and the error's name and message.
That is the same shape the Rust CLI writes through `tracing`.
Debug entries are dropped from production builds.

### Validation at the boundary

Every Tauri command result, certificate file, portal reply and CLI JSON result is parsed with a schema from `src/lib/schemas.ts` before a screen uses it.
A mismatch becomes an error that names the command and the field, instead of a wrong value on screen.

## The Rust backend

`src-tauri/src/main.rs` only adapts library functions to `#[tauri::command]`.
The rules live in the library and are tested in `src-tauri/tests/` on temporary folders.

| Module | Rules |
|---|---|
| `args.rs` | only `discover`, `wipe`, `backup` and `cert` may run; flags that skip the CLI's own guards are refused; `~` and `$HOME` are expanded |
| `cli.rs` | where the CLI binary is, output lines cut at 64 KiB on a character boundary, and `SECUREWIPE_DANGER=1` passed only to a destructive wipe |
| `files.rs` | folder listings, selection sizes that never follow links inside folders, the certificate file list, `~` in paths to open |
| `wipe.rs` | the typed `WIPE <serial>` confirmation, `CLEAR` or `PURGE` only, `/dev/` targets only, and system mount detection |

The webview may call only the folder dialog and shell open from the Tauri API, and a content security policy limits it to the app's own scripts and styles.
Everything else goes through the commands above.

## Tests

Tests never reach a real backend or disk.
`src/test/fakeTauri.ts` installs a fake through Tauri's official `mockIPC` hook before every test.
It answers commands from per-test handlers and delivers events to whatever subscribed, and its `cli()` helper streams output lines and the exit event before `run_securewipe` resolves, the way a fast CLI does.
`execute_destructive_wipe` is a canned handler in every test that uses it.
