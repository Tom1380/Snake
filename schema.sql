CREATE TABLE scores (
	username TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    score INTEGER NOT NULL,
    id serial,
    date TIMESTAMP NOT NULL DEFAULT NOW()
);