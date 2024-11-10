SELECT *
FROM chats
WHERE
    active = true
    AND id = ANY (
        array_remove(
            regexp_split_to_array(
                regexp_replace($1, '[^0-9,]', '', 'g'), -- Remove all non-numeric and non-comma characters, including whitespace
                ','
            ),
            ''
        )::BIGINT[]
    );