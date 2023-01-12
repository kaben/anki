INSERT INTO reviews as revlog (
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
