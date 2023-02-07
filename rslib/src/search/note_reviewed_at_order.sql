DROP TABLE IF EXISTS sort_order;
CREATE TABLE sort_order (
  pos integer PRIMARY KEY,
  nid integer NOT NULL UNIQUE
);
INSERT INTO sort_order (nid)
SELECT n.id
FROM reviews AS r
INNER JOIN cards AS c
ON r.cid == c.id
INNER JOIN notes AS n
ON c.nid == n.id
GROUP BY n.id
ORDER BY max(r.id);