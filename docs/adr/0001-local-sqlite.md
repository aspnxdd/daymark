# ADR 0001: local SQLite through rusqlite

**Status:** accepted for MVP · **Date:** 2026-10-25

## Context

A private desktop journal needs on-disk editing, timeline search and reliable entry-level updates. The user prefers SQLite and excludes SQLx compile-time database checks. There is no existing async application runtime.

## Decision

Use `rusqlite` with bundled SQLite, a dedicated private directory, one serialized connection, transactional writes, bound SQL and optimistic revisions. Use DELETE journaling and FULL synchronous commits. Treat the database as authoritative; no independently writable Markdown mirror. See [the alternatives comparison](../STORAGE.md).

## Consequences

No server or async runtime, simple error/commit ordering, deterministic native engine dependency. Every edit performs synchronous I/O and timeline refresh; this is acceptable only as a measured-small-journal MVP simplification. Move to a serialized worker when latency warrants it, retaining acknowledged saves and close gating. No encryption is implied; backup/recovery remain explicit user responsibilities.
