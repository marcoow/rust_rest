# Rust REST

This is a simply REST server written in Rust with
[axum](https://crates.io/crates/axum) and
[tokio-postgres](https://crates.io/crates/tokio-postgres).

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
tables get created etc. Install the `refinery_cli` crate and use that to
migrate the database:

```bash
cargo install refinery_cli
refinery migrate
```

### Running the server

```bash
cargo run
```

### Making requests with `curl`

#### Creating a new user

```bash
curl -X POST localhost:3000/users -H 'Content-Type: application/json' -d '{"name": "<name>"}'
```

#### Loading a single user

```bash
curl localhost:3000/users/<id>
```

#### Loading all users

```bash
curl localhost:3000/users
```

