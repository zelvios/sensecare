# SenseCare

IoT platform for monitoring indoor climate and handling service calls in hospital rooms.
- Rust API (axum + Diesel)
- SvelteKit frontend
- PostgreSQL
- ESP32 room nodes
- Docker Compose.

## Repository layout

```
sensecare/
├── backend/              Rust API – see backend/README.md
├── frontend/             SvelteKit app – see frontend/README.md
├── firmware/             ESP32 room node code – see firmware/README.md
├── docs/                 Architecture notes
├── scripts/dev.ps1       Command shortcuts for Docker Compose
├── .github/workflows/    CI
├── docker-compose.yml    db + api + web
├── .env.example          Template for .env
├── .gitignore            Files Git must never track
├── .gitattributes        Line ending rules
└── .editorconfig         Editor formatting rules
```

## Quick start

```powershell
git clone https://github.com/zelvios/sensecare sensecare
cd sensecare
Copy-Item .env.example .env       # then set POSTGRES_PASSWORD
.\scripts\dev.ps1 up
```

| Service | URL                          |
|---------|------------------------------|
| Web     ||
| API     ||
| DB      ||

## Root files

### `.gitattributes`

Forces LF line endings on every text file, on every operating system.

Everything in this repo eventually runs inside Linux (Docker containers, CI). Windows
Git would otherwise check files out with CRLF endings, which breaks shell scripts,
Dockerfiles and SQL files inside containers with cryptic errors. This file makes the
repository consistent.

If you cloned before this file existed and see `^M` or line-ending errors, normalise once:

```powershell
git rm -r --cached .
git reset --hard
```

Recommended global Git setting on Windows: `git config --global core.autocrlf false`.

### `.editorconfig`

Tells RustRover, VS Code, and most other editors to use LF, UTF-8, 2-space indentation
(4 for Rust) and to strip trailing whitespace. Works together with `.gitattributes`
so files are written correctly in the first place.

### `docker-compose.yml`

Defines the three services:

- **db** – PostgreSQL. Data is stored in the named volume `db-data`, so it
  survives `down`/`up`.
- **api** – built from `backend/Dockerfile`. Waits for the database to be healthy,
  runs migrations on startup, listens on 8080.
- **web** – built from `frontend/Dockerfile`. Waits for the API to be healthy,
  calls it over the internal network at `http://api:8080`, listens on 3000.

Only `api` and `web` are meant to be reachable from outside; `db` is internal.

### `scripts/dev.ps1`

Shortcuts for the Docker Compose commands. Run from the repo root:

| Command                    | Does                                                        |
|----------------------------|-------------------------------------------------------------|
| `.\scripts\dev.ps1 up`      | Build (if needed) and start all services in the background  |
| `.\scripts\dev.ps1 down`    | Stop and remove containers – **keeps** the database volume  |
| `.\scripts\dev.ps1 logs`    | Follow logs from all services (Ctrl+C to stop)              |
| `.\scripts\dev.ps1 rebuild` | Rebuild images from scratch, ignoring Docker's cache        |
| `.\scripts\dev.ps1 psql`    | Open a SQL shell in the database container                  |
| `.\scripts\dev.ps1 clean`   | Stop everything **and delete the database volume**          |

`up` is the default, so `.\scripts\dev.ps1` alone is the same as `up`.

If PowerShell refuses to run scripts, allow local scripts once:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

## Windows 11 notes

- **Toolchain**: WSL 2 + Docker Desktop (WSL 2 engine), Git for Windows, rustup with
  Visual Studio Build Tools (C++ workload), Node 22 LTS, pnpm.
- **Diesel CLI**: install the prebuilt binary rather than compiling it:
  `irm https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.ps1 | iex`
- **Smart App Control**: blocks freshly compiled Rust build scripts with
  `os error 4551`. Turn it off under Windows Security → App & browser control, or
  develop inside WSL 2.
- **ESP32 devices** must reach port 8080 on your PC: find the Wi-Fi IPv4 with
  `ipconfig`, set the network to Private, and allow the port through the firewall
  (admin PowerShell):
  `New-NetFirewallRule -DisplayName "SenseCare API" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow -Profile Private`
