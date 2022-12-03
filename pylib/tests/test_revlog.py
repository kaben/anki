# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

# coding: utf-8

from anki import revlog_pb2
from tests.shared import getEmptyCol

RevlogEntry = revlog_pb2.RevlogEntry
RevlogId = revlog_pb2.RevlogId


def make_test_notes(col):
    notes = [col.newNote() for i in range(10)]
    for i, note in enumerate(notes):
        note["Front"] = f"front {i}"
        note["Back"] = f"back {i}"
        col.addNote(note)
    return notes


def review_test_cards(col):
    card = col.sched.getCard()
    while card:
        col.sched.answerCard(card, 4)
        card = col.sched.getCard()


def make_and_review_test_cards(col):
    # Make a bunch of notes.
    notes = make_test_notes(col)
    # Review the corresponding cards.
    review_test_cards(col)
    return notes


def delete_all_cards(col):
    cids = col.find_cards("")
    col.reset()
    col.remove_cards_and_orphaned_notes(cids)


def test_get_revlog_entry():
    # Make a bunch of cards and notes, then review the cards.
    col = getEmptyCol()
    notes = make_and_review_test_cards(col)

    # Use a direct database command to set some feedback and some tags for the
    # last review.
    col_cids = col.find_cards("")
    col_cids.sort()
    last_cid = col_cids[-1]
    col.db.scalar(
        f"UPDATE revlog SET feedback = 'this is some feedback', tags = ' tag1 tag2 ' WHERE cid = {last_cid}"
    )

    # Read the database entry for the last review.
    revlog_rows = col.db.all(
        f"SELECT id,cid,usn,ease,ivl,lastIvl,factor,time,type,mod,feedback,tags FROM revlog WHERE cid = {last_cid}"
    )
    revlog_row = revlog_rows[0]
    revlog_id = revlog_row[0]
    # Get the same data (for the last review) via the backend command
    # `get_revlog_entry()`.
    revlog_entry = col._backend.get_revlog_entry(revlog_id)

    # Make sure the data matches.
    assert revlog_row[0] == revlog_entry.id
    assert revlog_row[1] == revlog_entry.cid
    assert revlog_row[2] == revlog_entry.usn
    assert revlog_row[3] == revlog_entry.button_chosen
    assert revlog_row[4] == revlog_entry.interval
    assert revlog_row[5] == revlog_entry.last_interval
    assert revlog_row[6] == revlog_entry.ease_factor
    assert revlog_row[7] == revlog_entry.taken_millis
    assert revlog_row[8] == revlog_entry.review_kind
    assert revlog_row[9] == revlog_entry.mtime_secs
    assert revlog_row[10] == revlog_entry.feedback
    assert revlog_row[11].split() == revlog_entry.tags

    delete_all_cards(col)


def test_update_revlog_entries():
    # Make a bunch of cards and notes, then review the cards.
    col = getEmptyCol()
    notes = make_and_review_test_cards(col)

    # Use a direct database command to set some feedback and some tags for the
    # last review.
    col_cids = col.find_cards("")
    col_cids.sort()
    last_cid = col_cids[-1]

    # Read the database entry for the last review.
    revlog_rows = col.db.all(
        f"SELECT id,cid,usn,ease,ivl,lastIvl,factor,time,type,mod,feedback,tags FROM revlog WHERE cid = {last_cid}"
    )
    revlog_row = revlog_rows[0]
    revlog_id = revlog_row[0]

    # row 10 is feedback and should be empty.
    assert revlog_row[10] == ""
    # row 11 is tags and should be empty.
    assert revlog_row[11].split() == []

    new_revlog_entry = RevlogEntry(
        id=revlog_id,
        cid=revlog_row[1],
        usn=revlog_row[2],
        button_chosen=revlog_row[3],
        interval=revlog_row[4],
        last_interval=revlog_row[5],
        ease_factor=revlog_row[6],
        taken_millis=revlog_row[7],
        review_kind=revlog_row[8],
        mtime_secs=revlog_row[9],
        feedback="this is some more feedback",
        tags=["tag3", "tag4"],
    )
    col._backend.update_revlog_entries(
        revlog_entries=[new_revlog_entry],
        skip_undo_entry=False,
    )
    revlog_rows = col.db.all(
        f"SELECT id,cid,usn,ease,ivl,lastIvl,factor,time,type,mod,feedback,tags FROM revlog WHERE id = {revlog_id}"
    )
    revlog_row = revlog_rows[0]

    # row 10 is feedback and should no longer be empty.
    assert revlog_row[10] == "this is some more feedback"
    # row 11 is tags and should no longer be empty.
    assert revlog_row[11].split() == ["tag3", "tag4"]

    delete_all_cards(col)
