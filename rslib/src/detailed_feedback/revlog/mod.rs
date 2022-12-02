// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{pb, revlog::RevlogEntry};

impl From<RevlogEntry> for pb::RevlogEntry {
    fn from(e: RevlogEntry) -> Self {
        pb::RevlogEntry {
            id: e.id.0,
            cid: e.cid.0,
            usn: e.usn.0,
            button_chosen: e.button_chosen as u32,
            interval: e.interval,
            last_interval: e.last_interval,
            ease_factor: e.ease_factor,
            taken_millis: e.taken_millis,
            review_kind: e.review_kind as i32,
            mtime_secs: e.mtime.0 as u32,
            feedback: e.feedback,
            tags: e.tags,
        }
    }
}
