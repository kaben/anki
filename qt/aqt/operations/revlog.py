# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from typing import Sequence

from anki.collection import OpChanges, OpChangesWithCount
from anki.revlog import RevlogEntry, RevlogId
from aqt.operations import CollectionOp
from aqt.qt import QWidget
from aqt.utils import tooltip, tr


def update_review(*, parent: QWidget, review: RevlogEntry) -> CollectionOp[OpChanges]:
    return CollectionOp(parent, lambda col: col.update_revlog_entry(review))


def remove_reviews(
    *,
    parent: QWidget,
    review_ids: Sequence[RevlogId],
) -> CollectionOp[OpChangesWithCount]:
    return CollectionOp(parent, lambda col: col.remove_reviews(review_ids)).success(
        lambda out: tooltip(tr.browsing_reviews_deleted(count=out.count)),
    )
