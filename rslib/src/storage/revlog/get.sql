SELECT id,
  cid,
  usn,
  ease,
  cast(ivl AS integer),
  cast(lastIvl AS integer),
  factor,
  time,
  type,
  mod,
  feedback,
  tags
FROM revlog