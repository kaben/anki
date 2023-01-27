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
Add-on for Anki 2.1: Random Seed Fields

Supplies a pair of fields:

- {{randseed-LastSuccessfulRevlogId:}} -- ID of last review for which user
clicked a 'success' button, e.g. 4 (easy), 3 (good), or 2 (hard), or "" if no
such review exists yet.
- {{randseed-Seed:}} -- The review ID above if it exists, otherwise the card ID.

The purpose is presenting math problems with randomized parameters. The idea is
that the same math problem will be presented each time the card is reviewed,
until the user solves the problem and clicks a 'success' button. Once solved, a
new variant of the problem with different parameters will be displayed
repeatedly until solved, and so on.

`success_buttons` are defined by an array, e.g. `[2,3,4]`, in `config.json`.
"""

import time
from typing import Any, Dict

from anki import hooks
from anki.template import TemplateRenderContext
from aqt import mw


def get_config(arg, fail=False):
    conf = mw.addonManager.getConfig(__name__)
    if conf:
        return conf.get(arg, fail)
    return fail

def on_field_filter(
    text: str, field: str, filter: str, context: TemplateRenderContext
) -> str:
    if not filter.startswith("randseed-"):
        return text

    # generate fields if not yet generated
    if "randseed_fields" not in context.extra_state:
        context.extra_state["randseed_fields"] = get_all_fields(context)
    rseed_fields: Dict[str, Any] = context.extra_state["randseed_fields"]

    # extract the requested field
    rseed, field = filter.split("-", maxsplit=1)
    value = str(rseed_fields.get(field, f"Unknown field: {field}"))
    return value

hooks.field_filter.append(on_field_filter)

def get_all_fields(context: TemplateRenderContext) -> Dict[str, Any]:
    fields: Dict[str, Any] = {}
    card = context.card()

    # Find id of most recent successful review. A successful review is one in
    # which the user pressed 2 (hard), 3 (good), or 4 (easy). An unsuccessful
    # review is one in which the user pressed 1 (again).
    success_buttons = tuple(get_config("success_buttons", (2,3,4,)))
    rids = mw.col.db.first(
      f"""
        SELECT id
          FROM revlog
          WHERE cid = {card.id} AND ease IN {success_buttons} 
          ORDER BY id DESC
          LIMIT 1
          ;
      """
    )
    # If such a review was found, use its ID as the random seed.
    # Otherwise, use the card ID.
    rid = rids[0] if rids else ""
    seed = rid or card.id

    fields["LastSuccessfulRevlogId"] = rid
    fields["Seed"] = seed

    return fields
