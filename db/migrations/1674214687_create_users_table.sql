CREATE TABLE users (
    id uuid PRIMARY KEY default gen_random_uuid(),
    name varchar(255) NOT NULL,
    token varchar(100) NOT NULL
);
