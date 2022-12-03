// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    ops::StateChanges,
    prelude::*,
    revlog::{undo::UndoableRevlogChange, RevlogEntry},
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
}

#[cfg(test)]
mod test {
    #[test]
    fn hello() {
        println!("hello from detailed_feedback::revlog::undo!");
        println!("FIXME@kaben: decide whether to unit test the functions in this file.");
    }
}
