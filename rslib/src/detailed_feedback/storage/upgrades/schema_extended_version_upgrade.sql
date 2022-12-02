ALTER TABLE col
ADD COLUMN extended_version INTEGER;
UPDATE col
SET extended_version = 202212011737;