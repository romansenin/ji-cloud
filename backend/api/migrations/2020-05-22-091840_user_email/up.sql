-- Your SQL goes here
ALTER TABLE users DROP COLUMN username;
ALTER TABLE users ADD COLUMN email TEXT NOT NULL;
ALTER TABLE users ADD COLUMN display_name VARCHAR(255) NOT NULL;