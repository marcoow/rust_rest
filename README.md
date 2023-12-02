# Rust REST

This is a simply REST server written in Rust with
[axum](https://crates.io/crates/axum) and
[sqlx](https://crates.io/crates/sqlx).

I'm using this to figure out a good structure for such a project and answer
questions like:

* where to put what (controllers, app state, models, etc, etc.)
* how to handle migrations (and how to run them)
* how to handle configuration for different environments

## Commands

### Setting up the database

The server requires a database to run. The connection string is read from the
[`.env`](.env) file and defaults to
`postgresql://rust_rest:rust_rest@localhost/rust_rest`. The repo contains a
Docker setup for a PostgreSql database that will work with that connection
string. Run via:

```bash
docker compose up
```

You can also change the connection string and connect to a different
(PostgreSql) database of course:

```bash
DATABASE_URL="postgresql://<user>:<password>@<host>/<database>"
```

Once the database is running, it needs to be migrated and seeded so that the
required tables get created and data gets inserted:

```bash
cargo db migrate
cargo db seed
```

The Docker setup already comes with pre-configured databases for development
and testing but they can also be dropped, created, and reset (drop, re-create,
migrate) via the command line if necessary (e.g. when switching between
branches with different database schemes):

```bash
cargo db drop
cargo db create
cargo db reset
```

#### Generating migrations

In order to generate a new migration, use the `generate` command:

```bash
cargo generate migration <migration-name>
```

### Running the server

```bash
cargo run
```

The log level can be set via the `RUST_LOG` env var (set one of `trace`,
`debug`, `info` (default), `warn`, `error`), e.g.:

```bash
RUST_LOG=trace cargo run
```

### Making requests with `curl`

#### Creating a new task

```bash
curl -X POST localhost:3000/tasks -H 'Authorization: 9974812642a36dbee625fa06b2463dbff832e17dcce3836dbb' -H 'Content-Type: application/json' -d '{"description": "do something"}'
```

#### Loading a single task

```bash
curl localhost:3000/tasks/<id>
```

#### Loading all tasks

```bash
curl localhost:3000/tasks
```

### Running the test

Migrate the test database:

```bash
cargo db migrate -e test
```

Then run the tests:

```bash
cargo test
```

