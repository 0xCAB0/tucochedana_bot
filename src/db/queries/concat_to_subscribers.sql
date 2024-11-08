-- This query updates the 'vehicles' table, appending the chat_id to the plate.
UPDATE vehicles
SET
    subscribers_ids = CONCAT(
        subscribers_ids,
        $1::text || ', '
    )
WHERE
    plate = $2;