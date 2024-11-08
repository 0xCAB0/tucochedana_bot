SELECT *
FROM chats
WHERE
    active = true
    AND id = ANY (
        string_to_array($1, ',')::BIGINT[]
    );