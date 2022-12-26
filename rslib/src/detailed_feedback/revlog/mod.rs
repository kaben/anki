// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod undo;

use crate::{
    card::CardId,
    pb,
    prelude::*,
    revlog::{RevlogEntry, RevlogId, RevlogReviewKind},
};

/// Conversion from `RevlogEntry` to `pb::RevlogEntry`, the latter being the Protocol Buffers
/// datatype defined in `proto/anki/revlog.proto`.
impl From<RevlogEntry> for pb::revlog::RevlogEntry {
    fn from(e: RevlogEntry) -> Self {
        pb::revlog::RevlogEntry {
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

/// Conversion from `pb::RevlogEntry` to `RevlogEntry`.
impl From<pb::revlog::RevlogEntry> for RevlogEntry {
    fn from(e: pb::revlog::RevlogEntry) -> Self {
        let review_kind = RevlogReviewKind::try_from(e.review_kind as u8)
            .or_invalid("invalid review kind")
            .unwrap();
        RevlogEntry {
            id: RevlogId(e.id),
            cid: CardId(e.cid),
            usn: Usn(e.usn),
            button_chosen: e.button_chosen as u8,
            interval: e.interval,
            last_interval: e.last_interval,
            ease_factor: e.ease_factor,
            taken_millis: e.taken_millis,
            review_kind,
            mtime: TimestampSecs(e.mtime_secs as i64),
            feedback: e.feedback,
            tags: e.tags,
        }
    }
}
