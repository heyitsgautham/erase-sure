# Contributing to SecureWipe

SecureWipe destroys data on real drives.
Read the safety rules first, then set up whichever part you are changing.

## Safety rules

- Never run a wipe, or a test script from `tests/*.sh`, against a drive that holds data you want.
- Tests must not touch block devices. Wipe logic is tested against temp files through a fake `CommandRunner` (see `core/src/wipe.rs` and `core/tests/common/mod.rs`).
- Never run `cargo test` as root. CI refuses to.
- Never commit keys, `.env` files or PEM files. `keys/` is gitignored, and `portal/keygen.py` creates a development pair.

## Setup

| Part | Setup | Checks to run before a pull request |
|---|---|---|
| `core/` | Rust 1.82 or newer | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |
| `portal/` | Python 3.11 or newer, `pip install --require-hashes -r requirements-lock.txt` | `ruff check .`, `ruff format --check .`, `python -m pytest --cov` |
| `ui/` | Node.js 20, `npm ci` | `npm run lint`, `npm run test:coverage`, `npm run build` |
| `ui/src-tauri/` | Rust, plus WebKitGTK 4.0 headers on Linux | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |

`cargo test` renders one PDF through `python3`, so activate the portal's virtualenv first.
`docker compose up --build` brings up the portal without any local setup.

## Dependencies

- Rust versions are locked in `core/Cargo.lock`; commit it with any `Cargo.toml` change.
- Python pins live in `portal/requirements.txt`. After editing it, regenerate the lock with `uv pip compile requirements.txt --universal --python-version 3.11 --generate-hashes -o requirements-lock.txt`.
- npm versions are locked in `ui/package-lock.json`; use `npm install <pkg>` so the lock updates.
- Dependabot opens weekly update pull requests for all three.

## Commits and pull requests

- One logical change per commit, with the tests that prove it in the same commit.
- Commit messages follow Conventional Commits, `type(scope): subject`, for example `fix(wipe): ...` or `test(portal): ...`.
- Changing a certificate field means updating `certs/schemas/`, the Rust builder in `core/src/cert.rs` and the portal tests together.
- Changes to signing, canonical JSON or the wipe guard rails need a second reviewer.

CI must be green before merge.
