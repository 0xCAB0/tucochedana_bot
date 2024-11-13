-- Insert test data into the `chats` table
-- Insert test data into the `chats` table with `user_id` as little-endian byte arrays
INSERT INTO
    chats (
        id,
        user_id,
        username,
        language_code,
        subscribed_vehicles
    )
VALUES (
        1,
        E'\\x6430000000000000',
        'user1',
        'en',
        'ABC123,DEF456,'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO
    chats (
        id,
        user_id,
        username,
        language_code,
        subscribed_vehicles
    )
VALUES (
        2,
        E'\\x6808bd0000000000'::bytea,
        'user2',
        'fr',
        'DEF456,'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO
    chats (
        id,
        user_id,
        username,
        language_code
    )
VALUES (
        3,
        E'\\x1cdc120000000000',
        'user3',
        'es'
    )
ON CONFLICT (id) DO NOTHING;
-- Insert test data into the `vehicles` table
INSERT INTO
    vehicles (plate, subscribers_ids)
VALUES ('ABC123', '1,')
ON CONFLICT (plate) DO NOTHING;

INSERT INTO
    vehicles (plate, subscribers_ids)
VALUES ('DEF456', '1,2,')
ON CONFLICT (plate) DO NOTHING;

INSERT INTO
    vehicles (plate)
VALUES ('GHI789')
ON CONFLICT (plate) DO NOTHING;