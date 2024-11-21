SELECT v.plate, COALESCE(COUNT(DISTINCT c.id), 0) AS subscriber_count
FROM vehicles v
    LEFT JOIN chats c ON TRIM(cast(c.id AS TEXT)) = ANY (
        string_to_array(TRIM(v.subscribers_ids), ',')
    )
WHERE
    v.plate = $1
    AND c.active = true
GROUP BY
    v.plate;