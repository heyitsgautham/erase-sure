---
applyTo:
  - "core/**"
---

# Copilot Instructions for Core (Rust)

- Use **Rust** only for code under /core.
- Implement CLI subcommands: `discover`, `backup`, `wipe`, `cert`.
- Integrate with Linux tools: `lsblk`, `hdparm`, `nvme-cli`, `smartctl`.
- Certificates must validate against `/certs/schemas/*.json`.
- Enforce NIST SP 800-88 policies (CLEAR, PURGE).
- Default wipe policy = PURGE; fallback to CLEAR with rationale logged in certificate.
- Logging: structured JSON logs with timestamps and step IDs.
- Tests required:
  - Unit tests (hashing, signing, schema validation).
  - Integration tests (backup → wipe → cert verification flow).
- Never wipe CRITICAL/system disks unless ISO mode + explicit override flag is set.
