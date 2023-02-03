# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations


class BrowserConfig:
    ACTIVE_CARD_COLUMNS_KEY = "activeCols"
    ACTIVE_NOTE_COLUMNS_KEY = "activeNoteCols"
    ACTIVE_REVIEW_COLUMNS_KEY = "activeReviewCols"

    CARDS_SORT_COLUMN_KEY = "sortType"
    NOTES_SORT_COLUMN_KEY = "noteSortType"
    REVIEWS_SORT_COLUMN_KEY = "reviewSortType"

    CARDS_SORT_BACKWARDS_KEY = "sortBackwards"
    NOTES_SORT_BACKWARDS_KEY = "browserNoteSortBackwards"
    REVIEWS_SORT_BACKWARDS_KEY = "browserReviewSortBackwards"

    # FIXME@kaben: this appears to be dead code. It might be used in plugins.
    @staticmethod
    # def active_columns_key(is_notes_mode: bool) -> str:
    def active_columns_key(mode: bool | str) -> str:
        # if is_notes_mode:
        #    return BrowserConfig.ACTIVE_NOTE_COLUMNS_KEY
        # return BrowserConfig.ACTIVE_CARD_COLUMNS_KEY
        if isinstance(mode, bool):
            is_notes_mode = mode
            if is_notes_mode:
                return BrowserConfig.ACTIVE_NOTE_COLUMNS_KEY
            return BrowserConfig.ACTIVE_CARD_COLUMNS_KEY
        else:
            if mode == "R":
                return BrowserConfig.ACTIVE_REVIEW_COLUMNS_KEY
            elif mode == "C":
                return BrowserConfig.ACTIVE_CARD_COLUMNS_KEY
            elif mode == "N":
                return BrowserConfig.ACTIVE_NOTE_COLUMNS_KEY
            else:
                return BrowserConfig.ACTIVE_NOTE_COLUMNS_KEY

    # FIXME@kaben: this appears to be dead code. It might be used in plugins.
    @staticmethod
    # def sort_column_key(is_notes_mode: bool) -> str:
    def sort_column_key(mode: bool | str) -> str:
        # if is_notes_mode:
        #    return BrowserConfig.NOTES_SORT_COLUMN_KEY
        # return BrowserConfig.CARDS_SORT_COLUMN_KEY
        if isinstance(mode, bool):
            is_notes_mode = mode
            if is_notes_mode:
                return BrowserConfig.NOTES_SORT_COLUMN_KEY
            return BrowserConfig.CARDS_SORT_COLUMN_KEY
        else:
            if mode == "R":
                return BrowserConfig.REVIEWS_SORT_COLUMN_KEY
            elif mode == "C":
                return BrowserConfig.CARDS_SORT_COLUMN_KEY
            elif mode == "N":
                return BrowserConfig.NOTES_SORT_COLUMN_KEY
            else:
                return BrowserConfig.NOTES_SORT_COLUMN_KEY

    # FIXME@kaben: this appears to be dead code. It might be used in plugins.
    @staticmethod
    # def sort_backwards_key(is_notes_mode: bool) -> str:
    def sort_backwards_key(mode: bool | str) -> str:
        # if is_notes_mode:
        #    return BrowserConfig.NOTES_SORT_BACKWARDS_KEY
        # return BrowserConfig.CARDS_SORT_BACKWARDS_KEY
        if isinstance(mode, bool):
            is_notes_mode = mode
            if is_notes_mode:
                return BrowserConfig.NOTES_SORT_BACKWARDS_KEY
            return BrowserConfig.CARDS_SORT_BACKWARDS_KEY
        else:
            if mode == "R":
                return BrowserConfig.REVIEWS_SORT_BACKWARDS_KEY
            elif mode == "C":
                return BrowserConfig.CARDS_SORT_BACKWARDS_KEY
            elif mode == "N":
                return BrowserConfig.NOTES_SORT_BACKWARDS_KEY
            else:
                return BrowserConfig.NOTES_SORT_BACKWARDS_KEY


class BrowserDefaults:
    CARD_COLUMNS = ["noteFld", "template", "cardDue", "deck"]
    NOTE_COLUMNS = ["noteFld", "note", "template", "noteTags"]
    REVIEW_COLUMNS = ["noteFld", "reviewedAt", "reviewNotes", "reviewTags"]
