# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from anki.collection import OpChanges
from anki.revlog import RevlogEntry
from aqt.operations import CollectionOp
from aqt.qt import QWidget


def update_review(*, parent: QWidget, review: RevlogEntry) -> CollectionOp[OpChanges]:
    return CollectionOp(parent, lambda col: col.update_revlog_entry(review))
