UPDATE vehicles
SET
    subscribers_ids = CONCAT(subscribers_ids, $1 || ',')
WHERE
    plate = $2;