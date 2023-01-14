// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    ops::StateChanges,
    prelude::*,
    revlog::{undo::UndoableRevlogChange, RevlogEntry, RevlogTags},
};

fn revlog_entry_differs_from_db(
    existing_revlog_entry: &mut RevlogEntry,
    revlog_entry: &mut RevlogEntry,
) -> bool {
    existing_revlog_entry != revlog_entry
}

impl Collection {
    /// The entry point for the backend `update_revlog_entries()` command, whih for each of the
    /// given review objects, updates the corresponding row of the revlog table. Note that one of
    /// the arguments indicates whether to update the undo stack.
    ///
    /// ## Arguments:
    ///
    /// - `revlog_entries`: the list of review objects.
    /// - `undoable`: whether to update the undo stack.
    pub(crate) fn update_revlog_entries_maybe_undoable(
        &mut self,
        revlog_entries: Vec<RevlogEntry>,
        undoable: bool,
    ) -> Result<OpOutput<()>> {
        if undoable {
            self.transact(Op::UpdateRevlogEntry, |col| {
                for mut revlog_entry in revlog_entries {
                    col.update_revlog_entry_inner(&mut revlog_entry)?;
                }
                Ok(())
            })
        } else {
            self.transact_no_undo(|col| {
                for mut revlog_entry in revlog_entries {
                    col.update_revlog_entry_inner(&mut revlog_entry)?;
                }
                Ok(OpOutput {
                    output: (),
                    changes: OpChanges {
                        op: Op::UpdateRevlogEntry,
                        changes: StateChanges {
                            revlog_entry: true,
                            ..Default::default()
                        },
                    },
                })
            })
        }
    }

    pub(crate) fn canonify_revlog_entry_tags(
        &mut self,
        entry: &mut RevlogEntry,
        usn: Usn,
    ) -> Result<()> {
        if !entry.tags.is_empty() {
            let tags = std::mem::take(&mut entry.tags);
            entry.tags = self.canonify_tags(tags, usn)?.0;
        }
        Ok(())
    }

    /// Called by `fn Collection::update_revlog_entries_maybe_undoable()` above, once for each
    /// review object. Undo-ably updates the database if there are any changes, after first
    /// cleaning up the entry's tags.
    pub(crate) fn update_revlog_entry_inner(&mut self, entry: &mut RevlogEntry) -> Result<()> {
        let mut existing_entry = self
            .storage
            .get_revlog_entry(entry.id)?
            .or_not_found(entry.id)?;
        if !revlog_entry_differs_from_db(&mut existing_entry, entry) {
            // nothing to do
            return Ok(());
        }

        let usn = self.usn()?;
        self.canonify_revlog_entry_tags(entry, usn)?;
        entry.usn = usn;

        self.update_revlog_entry_undoable(entry, existing_entry)
    }

    /// Called by `fn Collection::update_revlog_entry_inner()` above. Saves in the undo queue, and
    /// commits to DB.  No validation is done.
    ///
    /// This function is also used by `fn Collection::undo_revlog_change()` defined in
    /// `rslib/src/revlog/undo.rs`.
    ///
    /// This function calls `fn SqliteStorage::update_revlog_entry()` defined in
    /// `rslib/src/detailed_feedback/storage/revlog/mod.rs`.
    pub(crate) fn update_revlog_entry_undoable(
        &mut self,
        entry: &RevlogEntry,
        original: RevlogEntry,
    ) -> Result<()> {
        self.save_undo(UndoableRevlogChange::Updated(Box::new(original)));
        self.storage.update_revlog_entry(entry)?;
        Ok(())
    }

    pub(crate) fn update_revlog_tags_undoable(
        &mut self,
        tags: &RevlogTags,
        original: RevlogTags,
    ) -> Result<()> {
        self.save_undo(UndoableRevlogChange::TagsUpdated(Box::new(original)));
        self.storage.update_revlog_tags(tags)
    }

    pub(crate) fn remove_revlog_entries_undoable(
        &mut self,
        revlog_ids: &[RevlogId],
    ) -> Result<OpOutput<usize>> {
        let usn = self.usn()?;
        self.transact(Op::RemoveRevlogEntry, |col| {
            let revlog_entry_count = revlog_ids.len();
            for revlog_id in revlog_ids {
                col.remove_revlog_entry_inner(*revlog_id, usn)?;
            }
            Ok(revlog_entry_count)
        })
    }

    pub(crate) fn remove_revlog_entry_inner(
        &mut self,
        revlog_id: RevlogId,
        _usn: Usn,
    ) -> Result<()> {
        // FIXME@kaben: Add revlog graves entry or some other means of tracking deleted entries,
        // because the official Anki servers won't track this info. Must first add revlog graves
        // table, etc.
        let revlog_entry = self
            .storage
            .get_revlog_entry(revlog_id)?
            .or_not_found(revlog_id)?;
        self.remove_revlog_entry_undoable(revlog_entry)

        //Ok(())
    }
}

/* Anki isn't really set up for unit tests. Tests below aren't isolated from the collection or
 * storage systems. There's a lot of copy-paste here, which I'm okay with for now.
 */
#[cfg(test)]
mod test {
    use super::*;
    use crate::{collection::open_test_collection, types::Usn};

    #[test]
    fn revlog_entry_differs_from_db_with_unchanged_entry() -> Result<()> {
        let mut col = open_test_collection();
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        let mut note = nt.new_note();
        col.add_note(&mut note, DeckId(1))?;
        let post_answer = col.answer_again();
        let mut reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;

        let mut review = reviews[0].clone();

        assert!(!revlog_entry_differs_from_db(&mut review, &mut reviews[0]));

        Ok(())
    }

    #[test]
    fn revlog_entry_differs_from_db_with_changed_entry() -> Result<()> {
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

        assert!(revlog_entry_differs_from_db(&mut review, &mut reviews[0]));

        Ok(())
    }

    #[test]
    fn update_revlog_entry_undoable() -> Result<()> {
        let mut col = open_test_collection();
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        let mut note = nt.new_note();
        col.add_note(&mut note, DeckId(1))?;
        let post_answer = col.answer_again();
        let mut reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;

        let mut orig_review = reviews[0].clone();
        orig_review.usn = Usn(1234);
        col.storage.update_revlog_entry(&orig_review)?;

        let mut new_review = orig_review.clone();
        new_review.mtime = TimestampSecs(5678);
        new_review.feedback = "feedback".to_string();
        new_review.tags = ["fubar".to_string(), "fubaz".to_string()].to_vec();

        // Higher-level code changes usn, but this call shouldn't.
        col.update_revlog_entry_undoable(&new_review, orig_review)?;

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

    #[test]
    fn update_revlog_entry_inner_with_unchanged_entry_should_not_update_usn() -> Result<()> {
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

        // Low-level update to set usn.
        col.storage.update_revlog_entry(&review)?;

        // This call should NOT change usn because the review is unchanged.
        col.update_revlog_entry_inner(&mut review)?;

        reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;
        assert_eq!(reviews[0].usn, Usn(1234));

        Ok(())
    }

    #[test]
    fn update_revlog_entry_inner_with_changed_entry_should_update_usn() -> Result<()> {
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

        // Low-level update to set usn.
        col.storage.update_revlog_entry(&review)?;

        review.mtime = TimestampSecs(5678);
        review.feedback = "feedback".to_string();
        review.tags = ["fubar".to_string(), "fubaz".to_string()].to_vec();

        // This call should change usn because the review has changed.
        col.update_revlog_entry_inner(&mut review)?;

        reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;
        assert_eq!(reviews[0].usn, Usn(-1));
        assert_eq!(reviews[0].mtime, TimestampSecs(5678));
        assert_eq!(reviews[0].feedback, "feedback".to_string());
        assert_eq!(
            reviews[0].tags,
            ["fubar".to_string(), "fubaz".to_string()].to_vec()
        );

        Ok(())
    }

    #[test]
    fn update_revlog_entries_maybe_undoable_true_with_unchanged_entry_should_not_update_usn(
    ) -> Result<()> {
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

        // Low-level update to set usn.
        col.storage.update_revlog_entry(&review)?;

        let vec = vec![review];

        // This call should NOT change usn because the review is unchanged.
        col.update_revlog_entries_maybe_undoable(vec, true)?;

        reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;
        assert_eq!(reviews[0].usn, Usn(1234));

        Ok(())
    }

    #[test]
    fn update_revlog_entries_maybe_undoable_false_with_unchanged_entry_should_not_update_usn(
    ) -> Result<()> {
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

        // Low-level update to set usn.
        col.storage.update_revlog_entry(&review)?;

        let vec = vec![review];

        // This call should NOT change usn because the review is unchanged.
        col.update_revlog_entries_maybe_undoable(vec, false)?;

        reviews = col
            .storage
            .get_revlog_entries_for_card(post_answer.card_id)?;
        assert_eq!(reviews[0].usn, Usn(1234));

        Ok(())
    }
}
