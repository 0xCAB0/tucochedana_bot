INSERT INTO
    vehicles (
        plate,
        subscribers_ids,
        found_at
    )
VALUES ($1, $2, $3)
RETURNING
    *;