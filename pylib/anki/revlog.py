# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

# coding: utf-8

from anki import revlog_pb2
from anki.consts import *

RevlogEntry = revlog_pb2.RevlogEntry
RevlogId = NewType("RevlogId", int)
