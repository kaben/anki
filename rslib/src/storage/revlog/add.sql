INSERT INTO reviews AS revlog (
    id,
    cid,
    usn,
    ease,
    ivl,
    lastIvl,
    factor,
    time,
    type,
    feedback,
    tags,
    mod
  )
VALUES (
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    "",
    "",
    0
  )