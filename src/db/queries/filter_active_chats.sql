SELECT *
FROM chats
WHERE
    active = true
    AND id IN (
        SELECT UNNEST(
                string_to_array($1, ',')::BIGINT[]
            )
    );