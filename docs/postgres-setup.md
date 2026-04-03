# PostgreSQL Backend Setup

This guide explains how to run fake-fiori-server with PostgreSQL for persistent storage instead of in-memory JSON files.

## Prerequisites

- Docker and Docker Compose (recommended)
- OR a PostgreSQL 14+ server

## Quick Start

### 1. Start PostgreSQL

```bash
docker compose up -d
```

This starts a PostgreSQL 16 container with:
- User: `fiori`
- Password: `fiori`
- Database: `fiori`
- Port: `5433` (mapped from container's 5432)

### 2. Configure Environment

Create a `.env` file in the project root:

```bash
DATABASE_URL=postgres://fiori:fiori@localhost:5433/fiori
PORT=3000
RUST_LOG=info
```

Or export directly:

```bash
export DATABASE_URL=postgres://fiori:fiori@localhost:5433/fiori
```

### 3. Build with PostgreSQL Support

```bash
cargo build --release --features postgres
```

### 4. Run the Server

```bash
./target/release/fake-fiori-server
```

OR install globally:

```bash
cargo install --path . --features postgres
fake-fiori-server
```

You should see:

```
  Storage      : PostgreSQL
  Port         : 3000
```

## How It Works

### Storage Backend Selection

| Condition | Backend |
|-----------|---------|
| Built without `--features postgres` | In-Memory |
| Built with postgres, `DATABASE_URL` not set | In-Memory |
| Built with postgres, `DATABASE_URL` set, connection fails | In-Memory (fallback) |
| Built with postgres, `DATABASE_URL` set, connected | PostgreSQL |

### Database Schema

All entity data is stored in a single table:

```sql
CREATE TABLE entity_records (
    entity_set  TEXT    NOT NULL,      -- e.g. "Products", "Orders"
    key_value   TEXT    NOT NULL,      -- Primary key value
    is_active   BOOLEAN NOT NULL,      -- true=active, false=draft
    data        JSONB   NOT NULL,      -- Full entity record as JSON
    created_at  TIMESTAMPTZ NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (entity_set, key_value, is_active)
);
```

### Data Seeding

When an entity set is first accessed:
1. Server checks if records exist in PostgreSQL
2. If empty, seeds from `data/<EntitySet>.json`
3. Falls back to `mock_data()` if no JSON file exists

### Draft Support

The `is_active` column enables full OData V4 draft support:
- Active records: `is_active = true`
- Draft records: `is_active = false`
- Same key can have both active and draft versions

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | - | PostgreSQL connection string (required for PG backend) |
| `PORT` | `8000` | HTTP server port |
| `RUST_LOG` | `info` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |

## Connection String Format

```
postgres://[user]:[password]@[host]:[port]/[database]
```

Examples:
```bash
# Local Docker (port 5433 to avoid conflict with system postgres)
DATABASE_URL=postgres://fiori:fiori@localhost:5433/fiori

# Remote server
DATABASE_URL=postgres://admin:secret@db.example.com:5432/fiori_prod

# With SSL
DATABASE_URL=postgres://admin:secret@db.example.com:5432/fiori_prod?sslmode=require
```

## Troubleshooting

### "Connection refused"

```
ERROR: Failed to connect to PostgreSQL: connection refused
Storage: In-Memory (fallback)
```

**Solution**: Ensure PostgreSQL is running: `docker compose ps`

### "Database does not exist"

```
ERROR: database "fiori" does not exist
```

**Solution**: Create the database or use the provided docker-compose.yml which creates it automatically.

### "Permission denied"

**Solution**: Check that the user in `DATABASE_URL` has permissions on the database.

### Schema not created

The server auto-runs migrations from `migrations/001_create_entity_records.sql` on startup. If tables are missing, check the server logs for SQL errors.

## Switching Between Backends

You can switch backends without data loss:

1. **In-Memory → PostgreSQL**: Data is seeded from `data/*.json` files
2. **PostgreSQL → In-Memory**: Data persists in PostgreSQL; in-memory starts fresh from JSON files

The `commit()` operation writes to both PostgreSQL AND JSON files, so your `data/` folder stays synchronized.

## Production Considerations

- Use a managed PostgreSQL service (AWS RDS, Cloud SQL, etc.)
- Set `RUST_LOG=warn` to reduce log verbosity
- Configure connection pooling if needed (sqlx handles basic pooling)
- Back up the `entity_records` table for data recovery
