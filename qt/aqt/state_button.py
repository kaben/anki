# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from aqt import colors
from aqt.qt import *
from aqt.theme import theme_manager


class StateButton(QPushButton):
    def __init__(
        self,
        text: str = "",
        color: dict[str, str] = colors.ACCENT_DANGER | {},
        parent: QWidget = None,
    ) -> None:
        super().__init__(text=text, parent=parent)
        self._color = color
        self.setCheckable(True)
        self.setAutoExclusive(True)
        qconnect(self.toggled, self.update_color)
        super().setChecked(False)

    def update_color(self, checked):
        if checked:
            color = theme_manager.qcolor(self._color).name()
            self.setStyleSheet(f"QPushButton {{background-color: {color} }}")
        else:
            self.setStyleSheet("")
