// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{prelude::*, revlog::RevlogEntry, storage::SqliteStorage, tags::join_tags};
use rusqlite::params;

impl SqliteStorage {
    /// Called by `fn Collection::update_revlog_entry_undoable()` defined in
    /// `rslib/src/detailed_feedback/revlog/undo.rs`. This function issues SQL commands to update
    /// the `revlog` table row corresponding to the supplied `entry` object.
    pub(crate) fn update_revlog_entry(&self, entry: &RevlogEntry) -> Result<()> {
        let mut stmt = self.db.prepare_cached(include_str!("update.sql"))?;
        stmt.execute(params![
            entry.id.0,
            entry.cid.0,
            entry.usn.0,
            entry.button_chosen as u32,
            entry.interval,
            entry.last_interval,
            entry.ease_factor,
            entry.taken_millis,
            entry.review_kind as u8,
            entry.mtime.0,
            entry.feedback,
            join_tags(&entry.tags),
            entry.id,
        ])?;
        Ok(())
    }

    //pub(crate) fn get_revlog_ids_for_card(&self, cid: CardId) -> Result<Vec<RevlogId>> {
    //    self.db
    //        .prepare_cached("SELECT id FROM revlog WHERE revlog.cid=?")?
    //        .query_and_then([cid], |row| Ok(RevlogId(row.get(0)?)))?
    //        .collect()
    //}

    //pub(crate) fn get_revlog_ids_for_note(&self, nid: NoteId) -> Result<Vec<RevlogId>> {
    //    self.db
    //        .prepare_cached(
    //            "SELECT id FROM revlog INNER JOIN cards ON revlog.cid = cards.id WHERE cards.nid=?",
    //        )?
    //        .query_and_then([nid], |row| Ok(RevlogId(row.get(0)?)))?
    //        .collect()
    //}
}
