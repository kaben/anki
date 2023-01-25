// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::convert::TryFrom;

use rusqlite::params;
use rusqlite::types::FromSql;
use rusqlite::types::FromSqlError;
use rusqlite::types::ValueRef;
use rusqlite::Row;

use super::SqliteStorage;
use crate::error::Result;
use crate::prelude::*;
use crate::revlog::RevlogEntry;
use crate::revlog::RevlogId;
use crate::revlog::RevlogReviewKind;
use crate::tags::split_tags;

pub(crate) struct StudiedToday {
    pub cards: u32,
    pub seconds: f64,
}

impl FromSql for RevlogReviewKind {
    fn column_result(value: ValueRef<'_>) -> std::result::Result<Self, FromSqlError> {
        if let ValueRef::Integer(i) = value {
            Ok(Self::try_from(i as u8).map_err(|_| FromSqlError::InvalidType)?)
        } else {
            Err(FromSqlError::InvalidType)
        }
    }
}

fn row_to_revlog_entry(row: &Row) -> Result<RevlogEntry> {
    Ok(RevlogEntry {
        id: row.get(0)?,
        cid: row.get(1)?,
        usn: row.get(2)?,
        button_chosen: row.get(3)?,
        interval: row.get(4)?,
        last_interval: row.get(5)?,
        ease_factor: row.get(6)?,
        taken_millis: row.get(7).unwrap_or_default(),
        review_kind: row.get(8).unwrap_or_default(),

        // The following three fields were added to support a version of Anki for detailed review
        // feedback.
        mtime: row.get(9)?,
        feedback: row.get(10)?,
        tags: split_tags(row.get_ref_unwrap(11).as_str()?)
            .map(Into::into)
            .collect(),
    })
}

impl SqliteStorage {
    pub(crate) fn fix_revlog_properties(&self) -> Result<usize> {
        self.db
            .prepare(include_str!("fix_props.sql"))?
            .execute([])
            .map_err(Into::into)
    }

    pub(crate) fn clear_pending_revlog_usns(&self) -> Result<()> {
        self.db
            .prepare("update revlog set usn = 0 where usn = -1")?
            .execute([])?;
        Ok(())
    }

    /// Adds the entry, if its id is unique. If it is not, and `uniquify` is
    /// true, adds it with a new id. Returns the added id.
    /// (I.e., the option is safe to unwrap, if `uniquify` is true.)
    pub(crate) fn add_revlog_entry(
        &self,
        entry: &RevlogEntry,
        uniquify: bool,
    ) -> Result<Option<RevlogId>> {
        let mut available_id = entry.id.0;

        if uniquify {
            // FIXME@kaben: test uniquify works.
            while available_id < entry.id.0 + 1000 {
                match self.db.query_row(
                    "SELECT id FROM review_notes WHERE id == ? AND vis IN ('V', 'D')",
                    [available_id],
                    |row| row.get::<_, i64>(0),
                ) {
                    Err(e) => {
                        match e {
                            rusqlite::Error::QueryReturnedNoRows => {
                                //FIXME@kaben: remove debug output.
                                //println!("***** SqliteStorage.add_revlog_entry(): found
                                // available_id:{available_id:?}");
                                break;
                            }
                            _ => {
                                return Err(AnkiError::db_error(
                                    format!("error during search for available revlog ID: {e:?}"),
                                    crate::error::DbErrorKind::Other,
                                ))
                            }
                        }
                    }
                    _ => available_id += 1,
                }
            }
        }
        self.db
            .prepare_cached(include_str!("add.sql"))?
            .execute(params![
                available_id,
                entry.cid,
                entry.usn,
                entry.button_chosen,
                entry.interval,
                entry.last_interval,
                entry.ease_factor,
                entry.taken_millis,
                entry.review_kind as u8
            ])?;
        Ok(Some(RevlogId(entry.id.0)))
    }

    pub(crate) fn get_revlog_entry(&self, id: RevlogId) -> Result<Option<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(include_str!("get.sql"), " where id=?"))?
            .query_and_then([id], row_to_revlog_entry)?
            .next()
            .transpose()
    }

    /// Only intended to be used by the undo code, as Anki can not sync revlog
    /// deletions.
    pub(crate) fn remove_revlog_entry(&self, id: RevlogId) -> Result<()> {
        self.db
            .prepare_cached("delete from reviews where id = ?")?
            .execute([id])?;
        Ok(())
    }

    pub(crate) fn get_revlog_entries_for_card(&self, cid: CardId) -> Result<Vec<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(include_str!("get.sql"), " where cid=?"))?
            .query_and_then([cid], row_to_revlog_entry)?
            .collect()
    }

    // The following two functions were added to support a version of Anki for
    // detailed review feedback.
    pub(crate) fn get_revlog_entries_for_note(&self, nid: NoteId) -> Result<Vec<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(
                include_str!("get.sql"),
                " INNER JOIN cards ON revlog.cid = cards.id WHERE cards.nid=?"
            ))?
            .query_and_then([nid], row_to_revlog_entry)?
            .collect()
    }
    pub(crate) fn get_revlog_entries_with_tags(&self) -> Result<Vec<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(
                include_str!("get.sql"),
                " WHERE revlog.tags IS NOT NULL and revlog.tags !=''"
            ))?
            .query_and_then([], row_to_revlog_entry)?
            .collect()
    }

    pub(crate) fn get_revlog_entries_for_searched_cards_after_stamp(
        &self,
        after: TimestampSecs,
    ) -> Result<Vec<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(
                include_str!("get.sql"),
                " where cid in (select cid from search_cids) and id >= ?"
            ))?
            .query_and_then([after.0 * 1000], row_to_revlog_entry)?
            .collect()
    }

    pub(crate) fn get_revlog_entries_for_searched_cards(&self) -> Result<Vec<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(
                include_str!("get.sql"),
                " where cid in (select cid from search_cids)"
            ))?
            .query_and_then([], row_to_revlog_entry)?
            .collect()
    }

    pub(crate) fn get_all_revlog_entries(&self, after: TimestampSecs) -> Result<Vec<RevlogEntry>> {
        self.db
            .prepare_cached(concat!(include_str!("get.sql"), " where id >= ?"))?
            .query_and_then([after.0 * 1000], |r| row_to_revlog_entry(r).map(Into::into))?
            .collect()
    }

    pub(crate) fn studied_today(&self, day_cutoff: TimestampSecs) -> Result<StudiedToday> {
        let start = day_cutoff.adding_secs(-86_400).as_millis();
        self.db
            .prepare_cached(include_str!("studied_today.sql"))?
            .query_map([start.0, RevlogReviewKind::Manual as i64], |row| {
                Ok(StudiedToday {
                    cards: row.get(0)?,
                    seconds: row.get(1)?,
                })
            })?
            .next()
            .unwrap()
            .map_err(Into::into)
    }

    pub(crate) fn upgrade_revlog_to_v2(&self) -> Result<()> {
        self.db
            .execute_batch(include_str!("v2_upgrade.sql"))
            .map_err(Into::into)
    }
}
