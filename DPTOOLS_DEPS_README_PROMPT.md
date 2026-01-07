You are an expert Technical Writer and Rust Systems Architect.
Create a professional, high-impact `README.md` for a GitHub repository named **`dptools-deps`**.

## Context
This repository acts as the **Central Supply Depot** for the **DevPulse Ecosystem**.
**DevPulse** is a futuristic, local-first reactive state management system built on **Gwen3** (Rust), leveraging **Candle** and **GGUF** for local AI translation and middleware.
**Community**: [AngryViber.com](https://angryviber.com)
**Philosophy**: "Local First. Pure Rust. Zero Latency."

## Repository Purpose
To host versioned, pre-compiled binary packages (deps) for DevPulse tools (like `EzDbMigrate`).
This allows the `PulseManager` (our client-side hydration engine) to auto-update tools without core app updates.

## Technical Specifications (The Rules)
The README must strictly define how to contribute/upload new assets:

1.  **Release Tagging**: Semantic Versioning (e.g., `v1.0.0`).
2.  **Asset Structure**:
    *   Files must be **ZIP** archives.
    *   Filenames **MUST** include the target OS (e.g., `postgres-15-windows-x64.zip`).
3.  **The Supply Chain Menu (`manifest.json`)**:
    *   Must exist on the `main` branch: `deps/apps/ezdb/manifest.json`.
    *   **Structure**:
        ```json
        {
          "tool": "ezdb-migrator",
          "latest_version": "1.0.0",
          "packages": {
            "win32-x64": { "url": "...", "checksum": "...", "size_mb": 45 }
          }
        }
        ```
    *   *Rationale*: Defines the "Source of Truth" for client updates.

## Section Requirements
*   **Header**: Badges, DevPulse Logo (ASCII art or placeholder), "Official Dependency Source".
*   **"How It Works"**: Explain that `EzDbMigrate` queries `api.github.com/repos/devpulse/dptools-deps/releases/latest` to hydrate.
*   **Maintainer Guide**: Step-by-step on how to assume "God Mode" and repackage EDB binaries for this repo.
*   **Legal**: Disclaimer that we redistribute standard OSS binaries (Postgres license) for convenience.

## Tone
Technical, futuristic, "Cyberpunk Industrial". Usable by senior engineers but clearly documented.
