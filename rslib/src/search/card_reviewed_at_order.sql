DROP TABLE IF EXISTS sort_order;
CREATE TABLE sort_order (
  pos integer PRIMARY KEY,
  cid integer NOT NULL UNIQUE
);
INSERT INTO sort_order (cid)
SELECT c.id
FROM reviews AS r
INNER JOIN cards AS c
ON r.cid == c.id
GROUP BY c.id
ORDER BY max(r.id);