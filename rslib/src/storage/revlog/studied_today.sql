SELECT COUNT(),
  coalesce(sum(time) / 1000.0, 0.0)
FROM reviews AS revlog
WHERE id > ?
  AND type != ?