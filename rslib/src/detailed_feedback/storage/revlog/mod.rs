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

/* Anki isn't really set up for unit tests. Tests below aren't isolated from the collection or
 * storage systems. There's a lot of copy-paste here, which I'm okay with for now.
 */
#[cfg(test)]
mod test {
    use super::*;
    use crate::{collection::open_test_collection, types::Usn};

    #[test]
    fn update_revlog_entry() -> Result<()> {
        let mut col = open_test_collection();
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        let mut note = nt.new_note();
        col.add_note(&mut note, DeckId(1))?;
        let post_answer = col.answer_again();
        let mut reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;

        let mut review = reviews[0].clone();
        review.usn = Usn(1234);
        review.mtime = TimestampSecs(5678);
        review.feedback = "feedback".to_string();
        review.tags = ["fubar".to_string(), "fubaz".to_string()].to_vec();

        // Higher-level code changes usn, but this call shouldn't.
        col.storage.update_revlog_entry(&review)?;

        reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;
        assert_eq!(reviews[0].usn, Usn(1234));
        assert_eq!(reviews[0].mtime, TimestampSecs(5678));
        assert_eq!(reviews[0].feedback, "feedback".to_string());
        assert_eq!(
            reviews[0].tags,
            ["fubar".to_string(), "fubaz".to_string()].to_vec()
        );

        Ok(())
    }
}
