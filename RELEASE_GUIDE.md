# Pulse Driver Release Guide

To activate the "Smart Update" system, you must host the binaries on GitHub.

## 1. Create Repository
Create a public repository named `drivers` under your organization (default: `devpulse-tools`).
> **URL**: `github.com/devpulse-tools/drivers`

## 2. Prepare the Artifact
You need the **Plain Binaries (Zip)** for Windows x64.
**Direct Download**: [postgresql-15.6-1-windows-x64-binaries.zip](https://get.enterprisedb.com/postgresql/postgresql-15.6-1-windows-x64-binaries.zip)
*Mirror*: [EDB Binaries Page](https://www.enterprisedb.com/download-postgresql-binaries)

1.  Download the zip (~200MB).
2.  Extract it.
3.  Create a **NEW** zip named `postgres-15-windows-x64.zip`.
4.  Copy **ONLY** the following files into the root of your new zip (or inside a `bin/` folder):

### The Manifest (Required Files)
Copy these from your local `C:\Program Files\PostgreSQL\15\bin` or extracted Zip info:

**Executables:**
*   `pg_dump.exe`
*   `pg_restore.exe`
*   `psql.exe`

**Core Dependencies (The "Dll Hell" Fix):**
*   `libpq.dll` (Client Interface)
*   `libintl-8.dll` (Internationalization)
*   `libiconv-2.dll` (Iconv)
*   `libwinpthread-1.dll` (Threading)

**Security & Compression (Critical for Supabase):**
*   `libssl-3-x64.dll` (or `ssleay32.dll` depending on version)
*   `libcrypto-3-x64.dll` (or `libeay32.dll`)
*   `zlib1.dll`
*   `liblz4.dll` (Optional but recommended)
*   `libzstd.dll` (Optional but recommended)

> **Warning**: Missing `libssl` or `libcrypto` will cause "Connection Refused" errors when talking to Supabase.

## 3. Create Release
1.  Go to **Releases** -> **Draft a new release**.
2.  **Tag version**: `v15.0.0` (or similar).
3.  **Title**: "Pulse Core Drivers v15".
4.  **Upload**: Drag your `postgres-15-windows-x64.zip` into the assets.
5.  **Publish**.

## 4. Verification
The app checks `api.github.com/repos/devpulse-tools/drivers/releases/latest`.
It will find the zip, download it, and extract it to `%APPDATA%\DevPulse\bin\postgres-15`.
