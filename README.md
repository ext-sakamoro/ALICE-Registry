# ALICE Registry

Container and artifact registry SaaS with push/pull, layer diffing, and content-addressable search.

## Architecture

```
Frontend (Next.js 15)       API Gateway (port 8081)
  /dashboard/console   →    POST /api/v1/registry/push
  /                         POST /api/v1/registry/pull
                            POST /api/v1/registry/diff
                            POST /api/v1/registry/search
                            GET  /api/v1/stats
                                 │
          ┌──────────────────────┼──────────────────────┐
          ▼                      ▼                      ▼
   Manifest Store          Blob Store (CAS)       Search Index
  (OCI manifests,         (content-addressed     (label + text
   multi-arch)             layers, dedup)          full-text)
          │
   GC Worker (unreferenced blob cleanup)
```

## Features

| Feature | Description |
|---------|-------------|
| OCI Push/Pull | Standard OCI Image Manifest v1 with chunked upload |
| Layer Diffing | Content-addressable diff between any two tags |
| Content Search | Full-text and label-based search across repositories |
| Garbage Collection | Policy-driven GC for unreferenced blobs |
| Access Control | Per-repository RBAC with token-based auth |
| Multi-Format | OCI images, Helm charts, WASM modules, generic artifacts |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| GET | /api/v1/stats | Registry-wide statistics |
| POST | /api/v1/registry/push | Push a manifest and associate layers |
| POST | /api/v1/registry/pull | Pull a manifest by tag or digest |
| POST | /api/v1/registry/diff | Diff layers between two tags |
| POST | /api/v1/registry/search | Search repositories and artifacts |

### POST /api/v1/registry/push

```json
{
  "repository": "alice/runtime",
  "tag": "v1.2.0",
  "manifest": {
    "schema_version": 2,
    "media_type": "application/vnd.oci.image.manifest.v1+json",
    "config": { "digest": "sha256:abc123", "size": 1024 },
    "layers": [
      { "digest": "sha256:layer001", "size": 52428800 }
    ]
  }
}
```

### POST /api/v1/registry/pull

```json
{
  "repository": "alice/runtime",
  "reference": "v1.2.0",
  "resolve_layers": true
}
```

### POST /api/v1/registry/diff

```json
{
  "repository": "alice/runtime",
  "tag_a": "v1.1.0",
  "tag_b": "v1.2.0",
  "include_config_diff": true
}
```

### POST /api/v1/registry/search

```json
{
  "query": "alice runtime",
  "filters": { "namespace": "alice", "label": "env=production" },
  "limit": 20,
  "sort_by": "push_time"
}
```

## Quick Start

```bash
docker compose up -d
# API:      http://localhost:8081
# Frontend: http://localhost:3000
```

## License

AGPL-3.0-or-later
