# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
# type: ignore

import asyncio
import logging
import sys

from ipykernel.embed import embed_kernel
from ipykernel.kernelapp import IPKernelApp

from aqt.qt import *

log = logging.getLogger(__name__)
log.setLevel(logging.DEBUG)
ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
# create formatter
formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
# add formatter to ch
ch.setFormatter(formatter)
# add ch to logger
log.addHandler(ch)
# log.debug('debug message')
# log.info('info message')
# log.warning('warn message')
# log.error('error message')
# log.critical('critical message')


# Credit: `start_kernel()` is loosely based on code from PyXLL-Jupyter, GitHub
# repo https://github.com/pyxll/pyxll-jupyter. Accessed 5 January 2023.

if getattr(sys, "_ipython_kernel_running", None) is None:
    sys._ipython_kernel_running = False

if getattr(sys, "_ipython_app", None) is None:
    sys._ipython_app = False


class PushStdout:
    """Context manage to temporarily replace stdout/stderr."""

    def __init__(self, stdout, stderr):
        self.__stdout = stdout
        self.__stderr = stderr

    def __enter__(self):
        self.__orig_stdout = sys.stdout
        self.__orig_stderr = sys.stderr
        sys.stdout = self.__stdout
        sys.stderr = self.__stderr

    def __exit__(self, exc_type, exc_val, exc_tb):
        sys.stdout = self.__orig_stdout
        sys.stderr = self.__orig_stderr


def start_ipython_kernel():
    """starts the ipython kernel and returns the ipython app"""
    if sys._ipython_app and sys._ipython_kernel_running:
        return sys._ipython_app

    # The stdout/stderrs used by IPython. These get set after the kernel has started.
    ipy_stdout = sys.stdout
    ipy_stderr = sys.stderr

    # Patch IPKernelApp.start() so that it doesn't block.
    def _IPKernelApp_start(self):
        nonlocal ipy_stdout, ipy_stderr

        if self.poller is not None:
            self.poller.start()
        self.kernel.start()

        # Run the kernel event loop in a separate thread. On one hand, this
        # risks things like race conditions and deadlock. But on the other
        # hand, without a separate thread, the Anki UI becomes unresponsive
        # whenever a long-running operation is performed in IPythyon/Jupyter.
        # For my use case, I prefer the former situation.
        class Worker(QObject):
            def __init__(self, loop, ipy_kernel_app):
                super().__init__()
                self.loop = loop
                self.ipy_kernel_app = ipy_kernel_app

            def run_once(self):
                self.loop.stop()
                self.loop.run_forever()

            def run(self):
                sys._ipython_kernel_running = True
                while True:
                    try:
                        # Use the IPython stdout/stderr while running the kernel.
                        # Without this, stdout/stderr print to the console from
                        # which Anki was launched, rather than to
                        # IPython/Jupyter.
                        with PushStdout(ipy_stdout, ipy_stderr):
                            self.run_once()
                            if self.ipy_kernel_app.kernel.shell.exit_now:
                                log.debug(
                                    "IPython kernel stopping (%s)", self.connection_file
                                )
                                sys._ipython_kernel_running = False
                                return
                        QThread.msleep(25)
                    except:
                        log.error("Error polling Jupyter loop", exc_info=True)

        self.aloop = asyncio.get_event_loop()
        self.kernel_thread = QThread()
        self.worker = Worker(self.aloop, self)
        self.worker.moveToThread(self.kernel_thread)
        self.kernel_thread.started.connect(self.worker.run)
        self.kernel_thread.start()

    IPKernelApp.start = _IPKernelApp_start

    # IPython expects sys.__stdout__ to be set, and keep the original values to
    # be used after IPython has set its own.
    sys.__stdout__ = sys_stdout = sys.stdout
    sys.__stderr__ = sys_stderr = sys.stderr

    # Get or create the IPKernelApp instance and set the 'connection_dir' property
    if IPKernelApp.initialized():
        ipy = IPKernelApp.instance()
    else:
        # ipy = IPKernelApp.instance(local_ns={})
        ipy = IPKernelApp.instance()
        ipy.initialize([])

    # Call the API embed function, which will use the monkey-patched method above.
    embed_kernel(local_ns={})

    # Keep a reference to the kernel even if this module is reloaded.
    sys._ipython_app = ipy

    # Restore sys stdout/stderr and keep track of the IPython versions.
    ipy_stdout = sys.stdout
    ipy_stderr = sys.stderr
    sys.stdout = sys_stdout
    sys.stderr = sys_stderr

    # Patch user_global_ns so that it always references the user_ns dict.
    # (Without this, comprehensions raise `NameError: name ... is not defined`.
    # because comprehensions only work with variables in a global scope.)
    setattr(ipy.shell.__class__, "user_global_ns", property(lambda self: self.user_ns))

    # Patch ipapp so anything else trying to get a terminal app (e.g. ipdb)
    # gets our IPKernalApp..
    from IPython.terminal.ipapp import TerminalIPythonApp

    TerminalIPythonApp.instance = lambda: ipy

    return ipy
