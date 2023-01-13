# Rust REST

This is a simply REST server written in Rust with
[axum](https://crates.io/crates/axum) and
[tokio-postgres](https://crates.io/crates/tokio-postgres).

## Commands

### Running the server

```bash
cargo run
```

The server requires a database to run. By default it connects to
`postgresql://rust_rest:rust_rest@localhost/rust_rest` but you can pass a
connection string to a different (PostgreSql) database:

```bash
cargo run "postgresql://<user>:<password>@<host>/<database>"
```

### Making requests with `curl`

#### Creating a new user

```bash
curl -X POST localhost:3000/users -H 'Content-Type: application/json' -d '{"username": "<username>"}'
```

#### Loading a single user

```bash
curl localhost:3000/users/<id>
```

#### Loading all users

```bash
curl localhost:3000/users
```

