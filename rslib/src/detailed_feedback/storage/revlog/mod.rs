// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    prelude::*,
    revlog::{RevlogEntry, RevlogTags},
    storage::SqliteStorage,
    tags::join_tags,
};
use rusqlite::{params, Row};

impl RevlogTags {
    pub(crate) fn set_modified(&mut self, usn: Usn) {
        self.mtime = TimestampSecs::now();
        self.usn = usn;
    }
}

fn row_to_revlog_tags(row: &Row) -> Result<RevlogTags> {
    Ok(RevlogTags {
        id: row.get(0)?,
        mtime: row.get(1)?,
        usn: row.get(2)?,
        tags: row.get(3)?,
    })
}

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

    pub(crate) fn get_revlog_tags_by_predicate<F>(&mut self, want: F) -> Result<Vec<RevlogTags>>
    where
        F: Fn(&str) -> bool,
    {
        let mut query_stmt = self.db.prepare_cached(include_str!("get_tags.sql"))?;
        let mut rows = query_stmt.query([])?;
        let mut output = vec![];
        while let Some(row) = rows.next()? {
            let tags = row.get_ref_unwrap(3).as_str()?;
            if want(tags) {
                output.push(row_to_revlog_tags(row)?)
            }
        }
        Ok(output)
    }

    pub(crate) fn get_revlog_tags_by_id(
        &mut self,
        revlog_id: RevlogId,
    ) -> Result<Option<RevlogTags>> {
        self.db
            .prepare_cached(&format!("{} where id = ?", include_str!("get_tags.sql")))?
            .query_and_then([revlog_id], row_to_revlog_tags)?
            .next()
            .transpose()
    }

    pub(crate) fn update_revlog_tags(&mut self, revlog: &RevlogTags) -> Result<()> {
        self.db
            .prepare_cached(include_str!("update_tags.sql"))?
            .execute(params![revlog.mtime, revlog.usn, revlog.tags, revlog.id])?;
        Ok(())
    }

    //pub(crate) fn get_revlog_ids_for_card(&self, cid: CardId) -> Result<Vec<RevlogId>> {
    //    self.db
    //        .prepare_cached("SELECT id FROM reviews AS revlog WHERE revlog.cid=?")?
    //        .query_and_then([cid], |row| Ok(RevlogId(row.get(0)?)))?
    //        .collect()
    //}

    //pub(crate) fn get_revlog_ids_for_note(&self, nid: NoteId) -> Result<Vec<RevlogId>> {
    //    self.db
    //        .prepare_cached(
    //            "SELECT id FROM reviews AS revlog INNER JOIN cards ON revlog.cid = cards.id WHERE cards.nid=?",
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
