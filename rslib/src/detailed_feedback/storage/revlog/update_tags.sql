UPDATE reviews AS revlog
SET mod = ?,
  usn = ?,
  tags = ?
WHERE id = ?