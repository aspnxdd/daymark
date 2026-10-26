# Security

Daymark is an early local-only MVP, not an encrypted vault or security-certified product. Read [the threat model](docs/PRIVACY.md) and [backup/recovery guidance](docs/BACKUP.md).

## Reporting

Do not attach a real journal/database, private queries, keys, environment dump or raw crash report to a public issue. Reproduce with fictional text in a fresh temporary directory and provide a sanitized description, affected version and platform.

No private reporting mailbox or hosted security-report channel is configured in this repository yet. Contact the repository owner through an established private channel for a sensitive disclosure; if unavailable, do not publish exploit details or personal data while trying to find one.

## Before releases

Review dependencies and build scripts against current advisories; exercise persistence failure/recovery and platform permissions; verify runtime has no new network paths. Skill content review is not a transitive dependency audit. Standard SQLite remains plaintext, and secure deletion cannot erase backups/snapshots or guarantee physical-media erasure.
