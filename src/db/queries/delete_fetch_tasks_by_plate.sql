DELETE FROM fang_tasks
WHERE
    metadata ->> 'plate' = $1
    AND (metadata ->> 'type') = 'FetchTask'