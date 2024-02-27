## Copyright: Ankitects Pty Ltd and contributors
## License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
# from __future__ import annotations
#
# import aqt
# import aqt.main
# from aqt.qt import *
#
## from aqt.theme import theme_manager
# from aqt.utils import (  # getSaveFile,; maybeHideClose,; tooltip,; tr,
#    addCloseShortcut,
#    disable_help_button,
#    restoreGeom,
#    saveGeom,
# )
#
#
# class ReviewStats(QDialog):
#    """New deck stats."""
#
#    def __init__(self, mw: aqt.main.AnkiQt) -> None:
#        QDialog.__init__(self, mw, Qt.WindowType.Window)
#        mw.garbage_collect_on_dialog_finish(self)
#        self.mw = mw
#        self.name = "reviewStats"
#        self.form = aqt.forms.review_stats.Ui_Dialog()
#        f = self.form
#        self.setMinimumWidth(700)
#        disable_help_button(self)
#        restoreGeom(self, self.name, default_size=(800, 800))
#        f.setupUi(self)
#        addCloseShortcut(self)
#        self.form.web.hide_while_preserving_layout()
#        self.show()
#        self.refresh()
#        self.form.web.set_bridge_command(self._on_bridge_cmd, self)
#        self.activateWindow()
#
#    def reject(self) -> None:
#        self.form.web.cleanup()
#        self.form.web = None
#        saveGeom(self, self.name)
#        aqt.dialogs.markClosed("reviewStats")
#        QDialog.reject(self)
#
#    def closeWithCallback(self, callback: Callable[[], None]) -> None:
#        self.reject()
#        callback()
#
#    def _on_bridge_cmd(self, cmd: str) -> bool:
#        print(f'ReviewStats._on_ridge_cmd(cmd="{cmd}")')
#        if cmd.startswith("browserSearch"):
#            _, query = cmd.split(":", 1)
#            browser = aqt.dialogs.open("Browser", self.mw)
#            browser.search_for(query)
#
#        return False
#
#    def refresh(self) -> None:
#        self.form.web.load_ts_page("graphs")
