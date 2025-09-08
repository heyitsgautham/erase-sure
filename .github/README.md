# SecureWipe — Copilot Setup

This folder contains all **Copilot guidance files** for the SecureWipe project.  
They are split into three categories:

---

## 1. Global Instructions

- **copilot-instructions.md**
- Purpose: Defines **default rules** for the entire repository.
- Always loaded by Copilot no matter which file you are editing.

---

## 2. Path-Specific Instructions

- Location: `instructions/*.instructions.md`
- Purpose: Provides **module-specific rules**.
- Each file begins with `applyTo:` frontmatter to scope instructions to a path.

Current files:

- `core.instructions.md` → applies to `/core/**`
- `ui.instructions.md` → applies to `/ui/**`
- `portal.instructions.md` → applies to `/portal/**`

Example:  
If you edit `core/src/main.rs`, Copilot will apply both:

- Global rules from `copilot-instructions.md`
- Path-specific rules from `core.instructions.md`

---

## 3. Chat Prompts

- Location: `prompts/*.prompt.md`
- Purpose: Reusable **Copilot Chat prompts** you can trigger manually.
- Each prompt file is a template for a specific area of the project.

Current files:

- `core.prompt.md` → Rust erasure/backup engine
- `ui.prompt.md` → Tauri + React UI
- `portal.prompt.md` → FastAPI verification service

Usage in VS Code (with Copilot Chat):

- Type `/prompt core` in a chat window to load the `core.prompt.md`.
- Type `/prompt ui` to load the UI prompt.
- Type `/prompt portal` to load the portal prompt.

---

## Notes

- Do not edit schemas or PRD here; they live in `/docs` and `/certs/schemas`.
- Private keys must **never** be included in this repo.
- Keep instructions updated as the project evolves.

# end of file
