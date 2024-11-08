DELETE FROM fang_tasks
WHERE
    metadata ->> 'chat_id' = $1
    AND (metadata ->> 'type') = 'scheduled_fetch'