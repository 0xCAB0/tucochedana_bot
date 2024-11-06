DELETE FROM fang_tasks
WHERE
    metadata ->> 'profile_id' = $1
    AND (metadata ->> 'type') = 'FetchTask'