-- This query updates the 'chats' table, appending the plate to the subscribed_vehicles.
UPDATE chats
SET
    subscribed_vehicles = CONCAT(
        subscribed_vehicles,
        $2::text || ', '
    )
WHERE
    id = $1;