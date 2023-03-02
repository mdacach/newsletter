Email newsletter project based on [Zero to Production in Rust](https://www.zero2prod.com/index.html).

1. User subscribes to newsletter (user status is set to "pending confirmation")
2. User receives a confirmation email with generated token
3. User confirms email (user status is set to "confirmed")
4. User gets sent an email every time new issue drops

## Backend Development

### User authorization

- [OWASP guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- Password hashing + salt (argon2id hasher with work parameters)
- Protection against preimage attacks, dictionary attacks, timing attacks

### Databases

- Postgres with `sqlx` and support for offline mode (for docker building)
    - Compile-time correctness checks of queries
    - SQL for queries
    - Async support
- Database migrations
- Database transactions

### Deployment

- [Fly.io](https://fly.io/) Docker app deploy with Postgres cluster
- Zero-downtime deployments (incremental migrations and code updates)

### Docker

- Image size optimization (minimal runtimes, multi-stage builds)
- Image build run time optimization (caching dependencies binary with `cargo-chef`)

### Testing

- Unit testing (`reqwest`)
- Integration testing
- Property-based testing (`quickcheck` and `fake`)
- Continuous Integration (GitHub actions)
- Continuous Deployment (fly.io)

### Observability

- `log` logging,
- `tracing` spans
- `bunyan` formatting layer

### Type-driven development

- Parse, don't validate -> maintaining invariants with newtype pattern

### SMTP protocol for sending emails

- `lettre`

### Error handling

- Custom error generation with `thiserror` and `anyhow`
- Enum error types for control flow

### Actix-web

- Multithreaded execution
- Actix extractors with `serde`
- Actix middleware
- Actix cookies

### Configuration file parsing

- `config`
- Hierarchical configuration files
- Environment variables parsing

### Protection of sensitive data

- `secrecy`