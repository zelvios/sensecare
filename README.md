<h1 align="center">SenseCare</h1>
<h3 align="center">━━━━━━━━━  ❖  ━━━━━━━━━</h3>

<p align="center">IoT platform for indoor climate monitoring and service calls in hospital rooms.</p>

<!-- BADGES -->
<div align="center">

[![ci](https://img.shields.io/github/actions/workflow/status/zelvios/sensecare/ci.yml?branch=main&label=ci&labelColor=1A1B26&color=9ece6a&style=for-the-badge)](https://github.com/zelvios/sensecare/actions/workflows/ci.yml)
[![license](https://img.shields.io/github/license/zelvios/sensecare?color=FCA2AA&labelColor=1A1B26&style=for-the-badge)](https://github.com/zelvios/sensecare/blob/main/LICENSE)

[![issues](https://img.shields.io/github/issues/zelvios/sensecare?labelColor=1A1B26&color=e0af68&style=for-the-badge)](https://github.com/zelvios/sensecare/issues)
[![last commit](https://img.shields.io/github/last-commit/zelvios/sensecare?labelColor=1A1B26&color=7aa2f7&style=for-the-badge)](https://github.com/zelvios/sensecare/commits/main)
[![size](https://img.shields.io/github/repo-size/zelvios/sensecare?color=9ece6a&labelColor=1A1B26&style=for-the-badge)](https://github.com/zelvios/sensecare)

</div>

## Contents

- [Stack](#stack)
- [Repository layout](#repository-layout)
- [How to work on it](#how-to-work-on-it)
- [Quick start](#quick-start)
- [Tests](#tests)
- [Root files](#root-files)
  - [.env and .env.example](#env-and-envexample)
  - [.gitattributes](#gitattributes)
  - [.editorconfig](#editorconfig)
  - [docker-compose.yml](#docker-composeyml)
  - [scripts/dev.ps1](#scriptsdevps1)
- [Contributing](#contributing)
- [Windows 11 notes](#windows-11-notes)

## Stack

- Rust API (axum, Diesel with diesel-async)
- SvelteKit frontend (Svelte 5, Tailwind CSS)
- PostgreSQL 16
- ESP32 room nodes
- Docker Compose

## Repository layout

```
sensecare/
├── backend/              Rust API, see backend/README.md
├── frontend/             SvelteKit app, see frontend/README.md
├── firmware/             ESP32 room node code, see firmware/README.md
├── docs/                 Architecture, schema, acceptance tests
├── scripts/dev.ps1       Command shortcuts for Docker Compose
├── .github/workflows/    CI and PR checks
├── docker-compose.yml    db + api + web
├── .env.example          Template for .env
├── .gitattributes        Line ending rules
└── .editorconfig         Editor formatting rules
```

## How to work on it

Each part runs where it iterates fastest. The database is always the container.
The API and the frontend run natively while you edit them, and in Docker when you
want to verify the real build.

| Working on     | Run                                                        | Why                                              |
|----------------|------------------------------------------------------------|--------------------------------------------------|
| Backend        | `docker compose up -d db`, then `cargo run` in `backend/`  | 5 second rebuilds and a debugger                 |
| Frontend       | `docker compose up -d db api`, then `pnpm dev` in `frontend/` | hot reload, the API rarely changes meanwhile |
| Everything     | `.\scripts\dev.ps1 up`                                     | the real images, same as the server              |
| Firmware       | `docker compose up -d db api` and point the node at your PC's IP | see Windows notes for the firewall rule   |

Native runs read `backend/.env` and `frontend/.env`. Docker reads the root `.env`.
Each has an `.env.example` next to it.

After changing the API while the frontend is running against Docker:
`docker compose up -d --build api`.

## Quick start

```powershell
git clone https://github.com/zelvios/sensecare sensecare
cd sensecare
Copy-Item .env.example .env       # then set POSTGRES_PASSWORD
.\scripts\dev.ps1 up
```

| Service | URL                                    |
|---------|----------------------------------------|
| Web     | http://localhost:3000                  |
| API     | http://localhost:8080/swagger-ui       |
| Health  | http://localhost:8080/health           |
| DB      | localhost:5433 (dev only, user sensecare) |

The first start creates an `admin` account with the password from
`BOOTSTRAP_ADMIN_PASSWORD` in `.env`. Change it after logging in.

## Tests

```powershell
cd backend
cargo nextest run                  # unit + integration, needs the db container
cargo nextest run --test acceptance
```

Integration tests create a throwaway database per test, so they run in parallel.
Acceptance tests are named after the requirements (K1 to K13), see `docs/acceptance-tests.md`.

## Root files

### `.env` and `.env.example`

`.env` holds secrets and is git-ignored. `.env.example` is the committed template,
copy it and edit the copy.

### `.gitattributes`

Forces LF line endings on every text file, on every operating system. Everything in
this repo eventually runs inside Linux (Docker containers, CI). Windows Git would
otherwise check files out with CRLF endings, which breaks scripts, Dockerfiles and
SQL files inside containers. This file overrides each developer's local
`core.autocrlf`, so no personal Git configuration is required.

If you cloned before this file existed and see `^M` errors, normalise once:

```powershell
git rm -r --cached .
git reset --hard
```

### `.editorconfig`

Tells RustRover, VS Code and most other editors to use LF, UTF-8, 2-space indentation
(4 for Rust) and to strip trailing whitespace. Works together with `.gitattributes`
so files are written correctly in the first place.

### `docker-compose.yml`

Defines the three services:

- **db**: PostgreSQL. Data lives in the named volume `db-data`, so it survives
  `down` and `up`. Port 5433 is published for local tools. Remove that on a server.
- **api**: built from `backend/Dockerfile`. Waits for the database to be healthy,
  runs migrations on startup, listens on 8080.
- **web**: built from `frontend/Dockerfile`. Waits for the API to be healthy, calls
  it over the internal network at `http://api:8080`, listens on 3000.

### `scripts/dev.ps1`

Shortcuts for the Docker Compose commands. Run from the repo root:

| Command                     | Does                                                       |
|-----------------------------|------------------------------------------------------------|
| `.\scripts\dev.ps1 up`      | Build (if needed) and start all services in the background |
| `.\scripts\dev.ps1 down`    | Stop and remove containers, keeps the database volume      |
| `.\scripts\dev.ps1 logs`    | Follow logs from all services (Ctrl+C to stop)             |
| `.\scripts\dev.ps1 rebuild` | Rebuild images from scratch, ignoring Docker's cache       |
| `.\scripts\dev.ps1 psql`    | Open a SQL shell in the database container                 |
| `.\scripts\dev.ps1 clean`   | Stop everything and delete the database volume             |

`up` is the default, so `.\scripts\dev.ps1` alone is the same as `up`.

If PowerShell refuses to run scripts, allow local scripts once:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

## Contributing

Every change goes through a pull request into `main`. Branches are named
`type/short-description` (Conventional Branch), commits follow Conventional Commits,
and PRs are rebase-merged so each commit lands as written. CI runs fmt, clippy and the
test suite against a real Postgres.

## Windows 11 notes

- **Toolchain**: WSL 2 + Docker Desktop (WSL 2 engine), Git for Windows, rustup with
  Visual Studio Build Tools (C++ workload), Node 22 LTS, pnpm.
- **Diesel CLI**: install the prebuilt binary rather than compiling it:
  `irm https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.ps1 | iex`
- **Smart App Control** blocks freshly compiled Rust build scripts with
  `os error 4551`. Turn it off under Windows Security > App & browser control, or
  develop inside WSL 2.
- **ESP32 devices** must reach port 8080 on your PC: find the Wi-Fi IPv4 with
  `ipconfig`, set the network to Private, and allow the port through the firewall
  (admin PowerShell):
  `New-NetFirewallRule -DisplayName "SenseCare API" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow -Profile Private`
