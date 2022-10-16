SELECT
	*
FROM
	(
		SELECT
			MAX(score),
			username,
			date
		FROM
			scores
		WHERE
			difficulty = 0
		GROUP BY
			username
	) AS risultato
ORDER BY
	score DESC,
	date ASC;