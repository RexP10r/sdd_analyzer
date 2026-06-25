# SDD Navigator

A traceability scanner for Specification-Driven Development. Recursively finds `@req`
annotations in source code across six languages, computes per-requirement coverage
status, and exposes results via CLI reports and a REST API.

## Build

```sh
cargo build --release
```

## Usage (CLI)

```sh
# Full scan with zero-tolerance traceability check
sdd-coverage scan --requirements requirements.yaml --source . --strict

# Scan with test annotations shown
sdd-coverage scan --requirements requirements.yaml --source . --tests
```

## Usage (Server)

```sh
SDD_REQUIREMENTS_PATH=requirements.yaml \
SDD_SOURCE_PATH=. \
  ./target/release/sdd-coverage
```

Server listens on `0.0.0.0:3000`. Trigger a scan via `POST /scan`, then poll
`GET /scan` for status.

## API Endpoints

- `GET /healthcheck` — uptime check
- `GET /stats` — aggregate coverage summary
- `GET /requirements` — list all requirements (query: `?type=`, `?status=`, `?sort=id`, `?order=desc`)
- `GET /requirements/{id}` — requirement detail with annotations and tasks
- `GET /annotations` — list all annotations (query: `?type=impl|test`, `?orphans=true`)
- `GET /tasks` — list all tasks (query: `?status=`, `?orphans=true`, `?sort=id`)
- `POST /scan` — start background scan (202 Accepted)
- `GET /scan` — poll scan status (idle | scanning | completed | failed)
