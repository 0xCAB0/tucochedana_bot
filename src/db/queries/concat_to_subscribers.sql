UPDATE vehicles
SET
    subscribers_ids = CONCAT($1 || ', ')
WHERE
    plate = $2;