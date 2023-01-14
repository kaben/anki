// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    backend::Backend,
    pb::revlog::revlog_service::Service as RevlogService,
    pb::{self as pb},
    prelude::*,
    revlog::RevlogEntry,
};

impl RevlogService for Backend {
    /// Backend function. Returns a "Review" object (corresponding to a row in the revlog table)
    /// for the given review ID.
    fn get_revlog_entry(&self, input: pb::revlog::RevlogId) -> Result<pb::revlog::RevlogEntry> {
        let rid = input.rid.into();
        self.with_col(|col| {
            col.storage
                .get_revlog_entry(rid)
                .and_then(|opt| opt.or_not_found(rid))
                .map(Into::into)
        })
    }

    /// Backend function. For each of the given review objects, updates the corresponding row of
    /// the revlog table. Note that one of the arguments indicates whether to update the undo
    /// stack.
    ///
    /// ## `input` Arguments:
    ///
    /// - `input.revlog_entries`: the list of review objects.
    /// - `input.skip_uhdo_entry`: whether to skip updating the review stack; if `false` then it
    ///    should be possible to undo this update command.
    ///
    /// ## Returns:
    ///
    /// An `OpChanges` instance indicating that the operation `UpdateRevlogEntry` was
    /// performed, and that the operation changed a `revlog_entry`. (See `rslib/source/ops.rs` for
    /// more details.)
    fn update_revlog_entries(
        &self,
        input: pb::revlog::UpdateRevlogEntriesRequest,
    ) -> Result<pb::collection::OpChanges> {
        self.with_col(|col| {
            let revlog_entries = input
                .revlog_entries
                .into_iter()
                .map(Into::into)
                .collect::<Vec<RevlogEntry>>();
            col.update_revlog_entries_maybe_undoable(revlog_entries, !input.skip_undo_entry)
        })
        .map(Into::into)
    }

    fn remove_revlog_entries(
        &self,
        input: pb::revlog::RemoveRevlogEntriesRequest,
    ) -> Result<pb::collection::OpChangesWithCount> {
        self.with_col(|col| {
            let revlog_ids = input
                .revlog_ids
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();
            col.remove_revlog_entries_undoable(&revlog_ids)
        })
        .map(Into::into)

        //})?;
        //Ok(pb::collection::OpChangesWithCount {
        //    count: 0,
        //    changes: Some(pb::collection::OpChanges {
        //        ..Default::default()
        //    }),
        //})
    }
}
