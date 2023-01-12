CREATE TABLE IF NOT EXISTS last_inserted_row (table_name text, id integer);
INSERT INTO last_inserted_row (table_name, id) VALUES ('revlog', 0);
INSERT INTO last_inserted_row (table_name, id) VALUES ('review_notes', 0);
INSERT INTO last_inserted_row (table_name, id) VALUES ('reviews', 0);
DROP VIEW IF EXISTS reviews;
CREATE TABLE IF NOT EXISTS review_notes (
  id integer PRIMARY KEY,
  feedback TEXT DEFAULT "" NOT NULL,
  tags TEXT DEFAULT "" NOT NULL,
  mod INTEGER DEFAULT 0 NOT NULL,
  vis TEXT DEFAULT "" NOT NULL
);
CREATE VIEW IF NOT EXISTS reviews AS
SELECT revlog.id,
  revlog.cid,
  revlog.usn,
  revlog.ease,
  revlog.ivl,
  revlog.lastIvl,
  revlog.factor,
  revlog.time,
  revlog.type,
  review_notes.feedback,
  review_notes.tags,
  review_notes.mod,
  review_notes.vis
FROM revlog
  INNER JOIN review_notes ON revlog.id = review_notes.id
WHERE review_notes.vis IN ('D', 'V');
DROP TRIGGER IF EXISTS tr_aft_ins_on_revlog;
CREATE TRIGGER IF NOT EXISTS tr_aft_ins_on_revlog
AFTER
INSERT ON revlog BEGIN
UPDATE last_inserted_row SET id = 0 WHERE table_name = 'revlog';
UPDATE last_inserted_row SET id = 0 WHERE table_name = 'review_notes';
INSERT INTO review_notes
VALUES (NEW.id, "", "", 0, 'D');
UPDATE last_inserted_row SET id = NEW.id WHERE table_name = 'revlog';
UPDATE last_inserted_row SET id = NEW.id WHERE table_name = 'review_notes';
END;
DROP TRIGGER IF EXISTS tr_aftr_del_on_revlog;
DROP TRIGGER IF EXISTS tr_aft_del_on_revlog;
CREATE TRIGGER IF NOT EXISTS tr_aft_del_on_revlog
AFTER DELETE ON revlog BEGIN
DELETE FROM review_notes
WHERE id == OLD.id;
END;
DROP TRIGGER IF EXISTS tr_nstd_ins_on_reviews;
CREATE TRIGGER IF NOT EXISTS tr_nstd_ins_on_reviews INSTEAD OF
INSERT ON reviews BEGIN
UPDATE last_inserted_row SET id = 0 WHERE table_name = 'reviews';
INSERT INTO revlog
VALUES (
    NEW.id,
    NEW.cid,
    NEW.usn,
    NEW.ease,
    NEW.ivl,
    NEW.lastIvl,
    new.factor,
    NEW.time,
    NEW.type
  ) ON CONFLICT (id) DO
UPDATE
SET (cid, usn, ease, ivl, lastIvl, factor, time, type) = (
    NEW.cid,
    NEW.usn,
    NEW.ease,
    NEW.ivl,
    NEW.lastIvl,
    NEW.factor,
    NEW.time,
    NEW.type
  );
INSERT INTO review_notes
VALUES (NEW.id, NEW.feedback, NEW.tags, NEW.mod, 'D') ON CONFLICT (id) DO
UPDATE
SET (feedback, tags, mod, vis) = (
    NEW.feedback,
    NEW.tags,
    NEW.mod,
    (
      SELECT (
          CASE
            WHEN review_notes.vis == 'D' THEN 'D'
            ELSE 'V'
          END
        )
      FROM review_notes
      WHERE review_notes.id == NEW.id
    )
  );
UPDATE last_inserted_row SET id = NEW.id WHERE table_name = 'reviews';
END;
DROP TRIGGER IF EXISTS tr_nstd_upd_usn_on_reviews;
CREATE TRIGGER IF NOT EXISTS tr_nstd_upd_usn_on_reviews INSTEAD OF
UPDATE OF usn ON reviews BEGIN
UPDATE revlog
SET usn = NEW.usn
WHERE revlog.id == NEW.id;
UPDATE review_notes
SET vis = 'V'
WHERE (review_notes.id == NEW.id)
  AND (NEW.usn != -1);
END;
DROP TRIGGER IF EXISTS tr_nstd_upd_on_reviews;
CREATE TRIGGER IF NOT EXISTS tr_nstd_upd_on_reviews INSTEAD OF
UPDATE OF cid,
  ease,
  ivl,
  lastIvl,
  factor,
  time,
  type,
  feedback,
  tags,
  mod,
  vis ON reviews BEGIN
UPDATE revlog
SET (cid, ease, ivl, lastIvl, factor, time, type) = (
    NEW.cid,
    NEW.ease,
    NEW.ivl,
    NEW.lastIvl,
    NEW.factor,
    NEW.time,
    NEW.type
  )
WHERE id == NEW.id;
UPDATE review_notes
SET (feedback, tags, mod, vis) = (NEW.feedback, NEW.tags, NEW.mod, NEW.vis)
WHERE id == NEW.id;
END;
DROP TRIGGER IF EXISTS tr_nstd_del_on_reviews;
CREATE TRIGGER IF NOT EXISTS tr_nstd_del_on_reviews INSTEAD OF DELETE ON reviews BEGIN
DELETE FROM revlog
WHERE id IN (
    SELECT revlog.id
    FROM revlog
      INNER JOIN review_notes ON revlog.id == review_notes.id
    WHERE (revlog.id == OLD.id)
      AND (review_notes.vis == 'D')
  );
DELETE FROM review_notes
WHERE (review_notes.id == OLD.id)
  AND (review_notes.vis == 'D');
UPDATE review_notes
SET vis = ''
WHERE review_notes.id == OLD.id;
END;
UPDATE ankimath_info
SET version = 202301121442;
