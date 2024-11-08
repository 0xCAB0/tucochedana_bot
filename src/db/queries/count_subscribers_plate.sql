SELECT plate, COALESCE(
        array_length(
            string_to_array(subscribers_ids, ','), 1
        ), 0
    ) AS subscriber_count
FROM vehicles
WHERE
    plate = $1;