SELECT revlog.id,
  revlog.cid,
  revlog.usn,
  revlog.ease,
  cast(revlog.ivl AS integer),
  cast(revlog.lastIvl AS integer),
  revlog.factor,
  revlog.time,
  revlog.type
FROM revlog