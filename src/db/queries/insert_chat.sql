INSERT INTO chats (id , state , user_id, username, language_code) VALUES ($1 , $2 , $3, $4, $5) RETURNING *;
