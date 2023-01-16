# To-Do list for Anki-Math

- Features:
  - Local sync server:

    Need a local sync server handling `revlog` entry changes for the new
    `feedback`, `tags`, and `mod` columns. Anki-Math currently can't backup
    this information since it syncs `revlog` entry changes to the official Anki
    servers by omitting these new columns.

    Make sure that Anki-Math to can optionally sync to either kind of server.

  - Replaying review history:

    Need an option to select a card or cards in the browser, and replay the
    history of each selected card.

    By which I mean something like: make a temporary db with collection with
    decks corresponding to the selected cards; copy the collection and deck
    configurations/state over to the temporary db; for each selected card, make
    a corresponding temporary card in new state, then iterate over the reviews
    for the card, replaying each review (with same ID/timestamp, button, and
    time taken) for the remporary card; copy the final state of temporary card
    back to the original card.

  - Editing review history:

    Need to be able to manually add, change, and delete reviews.

    This is for entering review information after the fact, when studying was
    done without a device running Anki immediately at hand for recording
    reviews.

    Assuming mistakes will be made when manually entering this information,
    need to be able to correct mistakes. This includes changing the review
    timestamp and scoring, as well as deleting reviews. This presents two
    possible problems: review deletion, and scheduler consistency.

    The official Anki servers assume that reviews are never deleted, and that
    their timestamps never change. Thus deletion of a review from the local
    database won't sync with official Anki servers. Similarly, because the
    review's timestamp is used as its primary key in the database table,
    changing the timestamp amounts to deletion of the review from the database
    and insertion of a new review in its place, and again the deletion won't
    sync with the official Anki servers. I'm not sure how much of a problem
    this will cause, as it appears that locally deleted reviews will only be
    "undeleted" when a full sync is performed from an official Anki server db
    to the local one.

    The Anki schedulers aren't setup for recording reviews done in the past.
    Some of their code currently uses the computer's current date and time for
    any timestamp info, so this code may need to be modified to record reviews
    with timestamps in the past.

    The v3 scheduler does appear to partially support recording reviews in the
    past, but a problem arises when recording reviews out of order (scheduling
    a card review with a timestamp earlier than the card's most recent review).
    When this is done, the card state is updated, but not in a way that is
    consistent with the rewritten review history. It seems that making the
    history consistent requires replaying the card's history.

  - Math problem variation.

    Add a random-number generator that can be used to present variants of the
    same math problem. Seed the random number generator with the date of the
    last successful review, or the create date of the card if there are no
    reviews yet.

  - Make Math mode optional, so that Anki-Math can look and feel like standard
    Anki.

- Troubleshoot:
  - Figure out the cause of trouble including Jupyter in the Anki Python
    dependencies.
  - Need to troubleshoot the undo stack wrt replaying card histories.

- Refactoring:
  - `Table.switch_to_note/card/review_state(...)`: need to simplify and dry out.
  - Move code from _rslib/src/detailed_feedback/..._ to more appropriate
    locations.

- Need to test:
  - Importing
  - Exporting
  - After recent db refactor, anything having to do with SQL.
  - Verify whether standard Anki can use and syncronize the refactored AnkiMath
    db schema.

  - Verify whether `revlog` changes sync between AnkiMath and Anki apps via the
    Anki servers.

- Documentation:
  - Record info about the architecture of the AnkiMath db schema.
