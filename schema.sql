CREATE TABLE match (
	username TEXT NOT NULL,
	best_score_ever BIGINT NOT NULL,
    score BIGINT NOT NULL,
    numero_partita serial,
    date TIMESTAMP NOT NULL,
);

CREATE TABLE users (
	username TEXT NOT NULL,
	best_score_ever BIGINT NOT NULL,
    password bytea,
    id serial NOT NULL
);

