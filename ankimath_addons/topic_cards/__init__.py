# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

# Copyright: (c) Kaben Nanlohy <kaben.nanlohy@gmail.com>
# 
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option) any
# later version.
# 
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
# details.
"""
Add-on for Anki 2.1: Topic Cards
"""

import re
import time
from typing import Any, Dict, Literal

from anki import cards, collection, errors, hooks
from anki.template import TemplateRenderContext
from aqt import gui_hooks, mw, reviewer
from aqt.operations.scheduling import bury_cards
from aqt.utils import tooltip

topic_card_re = re.compile(r'topic-card-(?P<flashcard_field>.+)-from-(?P<deckname_field>.+)$')

def get_config(arg, fail=False):
    conf = mw.addonManager.getConfig(__name__)
    if conf:
        return conf.get(arg, fail)
    return fail

topic_cards_state: Dict[str, Any] = {}

def on_field_filter(
    text: str, field: str, filter: str, context: TemplateRenderContext
) -> str:
    topic_card_match = topic_card_re.fullmatch(filter)
    if not topic_card_match:
        return text

    #print(f'***** topic_cards.on_field_filter(): text: {text}')
    #print(f'----- topic_cards.on_field_filter(): field: {field}')
    #print(f'----- topic_cards.on_field_filter(): filter: {filter}')
    #print(f'----- topic_cards.on_field_filter(): context: {context}')
    #print(f'----- topic_cards.on_field_filter(): mw.reviewer.state: {mw.reviewer.state}')

    state = topic_cards_state.setdefault(
        context.card().id,
        dict(
            fields = dict()
        )
    )
    #print(f'----- topic_cards.on_field_filter(): state: {state}')
    if len(topic_cards_state) > 1:
        print(f"WARNING: topic_cards_state dict has too many entries. Clearing.")
        topic_cards_state.clear()
        topic_cards_state[context.card().id] = state

    flashcard_field, deckname_field = topic_card_match.groups()

    #print(f'----- topic_cards.on_field_filter(): flashcard_field: {flashcard_field}')
    #print(f'----- topic_cards.on_field_filter(): deckname_field: {deckname_field}')

    if 'flashcard_state' in state:
        flashcard_state = state['flashcard_state']
    else:
        note = context.note()

        if deckname_field not in note:
            raise LookupError(f'cannot find field "{deckname_field}" in topic note.')

        flashcard_deckname = note[deckname_field]
        if '::' in flashcard_deckname:
            unparented_flashcard_deckname = flashcard_deckname.split('::')[-1]
            short_flashcard_deckname = f'...::{unparented_flashcard_deckname}'
        else:
            short_flashcard_deckname = flashcard_deckname
        flashcard_deck_id = context.col().decks.id_for_name(flashcard_deckname)

        if flashcard_deck_id is None:
            raise errors.NotFoundError(f'cannot find deck named "{flashcard_deckname}".', None, None, None)

        current_deck_id = context.col().decks.get_current_id()
        try:
            context.col().decks.set_current(flashcard_deck_id)
            context.col().reset()
            queued_cards = context.col().sched.get_queued_cards()
            # FIXME@kaben: if no cards in queue, bury topic card.
            if len(queued_cards.cards) < 1:
                #if mw.reviewer.state is not None:
                #    gui_hooks.reviewer_will_bury_card(context.card().id)
                #    bury_cards(parent=mw, card_ids=[context.card().id],).success(
                #        lambda _: tooltip(f'Topic card buried because flashcard deck "{short_flashcard_deckname}" has empty queue.')
                #    ).run_in_background()
                return f'(Flashcard deck "{short_flashcard_deckname}" has empty queue)'
            queued_card = queued_cards.cards[0]
            flashcard = cards.Card(context.col(), queued_card.card.id)
            flashcard.load()
            flashcard.start_timer()
        finally:
            context.col().decks.set_current(current_deck_id)

        flashcard_state = dict(flashcard = flashcard)
        state['flashcard_state'] = flashcard_state

    #print(f'----- topic_cards.on_field_filter(): flashcard_state: {flashcard_state}')

    flashcard = flashcard_state['flashcard']
    #print(f'----- topic_cards.on_field_filter(): flashcard: {flashcard}')
    if flashcard_field == 'question':
        if 'question' in state['fields']:
            value = state['fields']['question']
        else:
            if 'rendered_output' in flashcard_state:
                rendered_output = flashcard_state['rendered_output']
            else:
                rendered_output = flashcard.render_output()
                flashcard_state['rendered_output'] = rendered_output
            value = rendered_output.question_text
            state['fields']['question'] = value
    elif flashcard_field == 'answer':
        if 'answer' in state['fields']:
            value = state['fields']['answer']
        else:
            if 'rendered_output' in flashcard_state:
                rendered_output = flashcard_state['rendered_output']
            else:
                rendered_output = flashcard.render_output()
                flashcard_state['rendered_output'] = rendered_output
            value = rendered_output.answer_text
            state['fields']['answer'] = value
    elif flashcard_field == 'css':
        if 'css' in state['fields']:
            value = state['fields']['css']
        else:
            if 'rendered_output' in flashcard_state:
                rendered_output = flashcard_state['rendered_output']
            else:
                rendered_output = flashcard.render_output()
                flashcard_state['rendered_output'] = rendered_output
            value = rendered_output.css
            state['fields']['css'] = value
    else:
        flashcard_note = flashcard.note()
        if flashcard_field not in flashcard_note:
            raise LookupError(f'cannot find field "{flashcard_field}" in flashcard note.')
        value = flashcard_note[flashcard_field]

    #print(f'----- topic_cards.on_field_filter(): value: {value}')
    return value


hooks.field_filter.append(on_field_filter)

def on_schedv2_did_answer_review_card(
    card: cards.Card, ease: int, early: bool,
) -> None: 
    #print(f"***** topic_cards.on_schedv2_did_answer_review_card: card: {card}")
    #print(f"----- topic_cards.on_schedv2_did_answer_review_card: ease: {ease}")
    #print(f"----- topic_cards.on_schedv2_did_answer_review_card: early: {early}")
    flashcard = topic_cards_state[card.id]['flashcard_state']['flashcard']
    mw.col.sched.answerCard(flashcard, ease)
    del topic_cards_state[card.id]

hooks.schedv2_did_answer_review_card.append(on_schedv2_did_answer_review_card)


def on_reviewer_did_answer_card(
    reviewer: reviewer.Reviewer,
    card: cards.Card,
    ease: Literal[1, 2, 3, 4],
) -> None:
    #print(f"***** topic_cards.on_reviewer_did_answer_card: reviewer: {reviewer}")
    #print(f"***** topic_cards.on_reviewer_did_answer_card: card: {card}")
    #print(f"***** topic_cards.on_reviewer_did_answer_card: ease: {ease}")
    flashcard = topic_cards_state[card.id]['flashcard_state']['flashcard']
    current_deck_id = mw.col.decks.get_current_id()
    mw.col.decks.set_current(flashcard.did)
    mw.col.sched.answerCard(flashcard, ease)
    mw.col.reset()
    mw.col.decks.set_current(current_deck_id)
    del topic_cards_state[card.id]

gui_hooks.reviewer_did_answer_card.append(on_reviewer_did_answer_card)


def on_reviewer_will_bury_card(
    id: int,
) -> None:
    print(f"***** topic_cards.on_reviewer_will_bury_card: id: {id}")
    if id in topic_cards_state:
        del topic_cards_state[id]

gui_hooks.reviewer_will_bury_card.append(on_reviewer_will_bury_card)


def on_state_did_undo(
    changes: collection.OpChangesAfterUndo,
) -> None:
    print(f"***** topic_cards.on_state_did_undo: changes: {changes}")

gui_hooks.state_did_undo.append(on_state_did_undo)