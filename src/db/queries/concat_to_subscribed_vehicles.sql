-- This query updates the 'chats' table, appending the plate to the subscribed_vehicles.
UPDATE chats
SET
    subscribed_vehicles = CONCAT($1 || ', ')
WHERE
    id = $2;