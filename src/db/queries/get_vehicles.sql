-- $1 should be the string containing the comma-separated plates (e.g., "ABC123,DEF456,GHI789").
-- string_to_array($1, ',') converts this string into an array {ABC123, DEF456, GHI789}.
-- ANY() checks if plate is in this array.

SELECT * FROM vehicles WHERE plate = ANY (string_to_array($1, ','));