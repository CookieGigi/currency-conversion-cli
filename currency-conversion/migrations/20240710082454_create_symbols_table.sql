-- Add migration script here
CREATE TABLE symbols(
id uuid NOT NULL,
PRIMARY KEY(id),
code TEXT NOT NULL UNIQUE,
name TEXT NOT NULL
);
