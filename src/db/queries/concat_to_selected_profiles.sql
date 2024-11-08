UPDATE vehicles
SET
    chat_ids = CONCAT(chat_ids, $1::text)
WHERE
    id = $2
    -- UPDATE chats SET selected_profiles = CONCAT(selected_profiles, '2,') WHERE id = 1361356382 AND user_id = E'\\x5ea6245100000000'
    -- It also worked without user_id