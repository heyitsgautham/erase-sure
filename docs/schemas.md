# SecureWipe — Certificate Schemas (Reference)

This document provides the **reference definitions** for Backup and Wipe Certificates.  
These must always match the machine-readable JSON Schema files stored in `/certs/schemas/`.

---

## Backup Certificate (Reference)

**Schema file**: `/certs/schemas/backup_schema.json`  
**Purpose**: Records metadata about a completed backup before wipe.

### Required Fields

- `cert_type`: must be `"backup"`.
- `cert_id`: unique identifier, e.g., `bkp_2025-09-08_001`.
- `device`: object with:
  - `model` (string)
  - `serial` (string, may be masked if needed)
  - `bus` (string: SATA, NVMe, USB)
  - `capacity_bytes` (integer)
- `files_summary`: object with:
  - `count` (integer, number of files backed up)
  - `personal_bytes` (integer, total size of personal data in bytes)
- `destination`: object with:
  - `type` (string: usb, nas, cloud, other)
  - `label` (string, e.g., volume name)
  - `fs` (string, e.g., ext4, exfat)
- `crypto`: object with:
  - `alg` = `"AES-256-CTR"`
  - `manifest_sha256` (64-char hex string of manifest file)
- `created_at`: ISO 8601 timestamp with timezone.
- `signature`: object with:
  - `alg` = `"Ed25519"`
  - `sig` (base64-encoded signature of the certificate JSON)
  - `pubkey_id` (string, must be `"sih_root_v1"`)

---

## Wipe Certificate (Reference)

**Schema file**: `/certs/schemas/wipe_schema.json`  
**Purpose**: Records metadata about the secure wipe process.

### Required Fields

- `cert_type`: must be `"wipe"`.
- `cert_id`: unique identifier, e.g., `wp_2025-09-08_045`.
- `device`: object with:
  - `model` (string)
  - `serial` (string, may be masked if needed)
  - `bus` (string: SATA, NVMe, USB)
  - `capacity_bytes` (integer)
- `policy`: object with:
  - `nist_level` (string: CLEAR, PURGE, DESTROY)
  - `method` (string describing actual method, e.g., `nvme_sanitize_block_erase`)
- `hpa_dco`: object (optional):
  - `cleared` (boolean)
  - `commands` (array of strings run to clear HPA/DCO)
- `commands`: array of command objects:
  - `cmd` (string of executed command)
  - `exit` (integer exit code)
  - `ms` (integer duration in milliseconds)
- `verify`: object with:
  - `strategy` (string: random_sectors, full_readback, controller_status)
  - `samples` (integer, number of verification samples)
  - `failures` (integer, number of failed samples)
- `linkage`: object:
  - `backup_cert_id` (string referencing prior backup certificate ID)
- `created_at`: ISO 8601 timestamp with timezone.
- `signature`: object with:
  - `alg` = `"Ed25519"`
  - `sig` (base64-encoded signature)
  - `pubkey_id` (string, must be `"sih_root_v1"`)

---

## Notes

- All numeric values are integers.
- All timestamps must be RFC 3339/ISO 8601 with explicit timezone (e.g., `2025-09-08T12:45:10+05:30`).
- `signature.sig` always covers the full JSON (excluding itself).
- Certificates must be valid against their schema in `/certs/schemas/` before acceptance.
- PDF certificates embed the signed JSON as an attachment, ensuring integrity.

---

## Example IDs

- Backup: `bkp_2025-09-08_001`
- Wipe: `wp_2025-09-08_045`

---

# End of File
