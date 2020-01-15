CREATE TABLE scores (
	username TEXT NOT NULL,
	best_score_ever BIGINT NOT NULL,
    difficulty INTEGER NOT NULL,
    score INTEGER NOT NULL,
    numero_partita serial,
    date TIMESTAMP NOT NULL DEFAULT NOW()
);