# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

# coding: utf-8

from tests.shared import getEmptyCol


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
