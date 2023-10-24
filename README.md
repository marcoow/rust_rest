# Rust REST

This is a simply REST server written in Rust with
[axum](https://crates.io/crates/axum) and
[tokio-postgres](https://crates.io/crates/tokio-postgres).

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

Once the database is running, it needs to be migrated so that the required
tables get created etc.:

```bash
cargo run --bin db migrate
```

### Running the server

```bash
cargo run
```

### Making requests with `curl`

#### Creating a new task

```bash
curl -X POST localhost:3000/task -H 'Authorization: Bearer secr3t!' -H 'Content-Type: application/json' -d '{"description": "do something"}'
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
cargo run --bin db migrate -e test
```

Then run the tests:

```bash
cargo test
```

