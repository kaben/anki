from aqt.qt import *
from ipykernel.kernelapp import IPKernelApp
from ipykernel.embed import embed_kernel
from zmq.eventloop import ioloop

import asyncio
import logging
import sys
import threading
import trace

_log = logging.getLogger(__name__)
_log.setLevel(logging.DEBUG)
ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)

# create formatter
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')

# add formatter to ch
ch.setFormatter(formatter)

# add ch to logger
_log.addHandler(ch)

# 'application' code
_log.debug('debug message')
_log.info('info message')
_log.warning('warn message')
_log.error('error message')
_log.critical('critical message')


class thread_with_trace(threading.Thread):
  def __init__(self, *args, **keywords):
    threading.Thread.__init__(self, *args, **keywords)
    self.killed = False
 
  def start(self):
    self.__run_backup = self.run
    self.run = self.__run     
    threading.Thread.start(self)
 
  def __run(self):
    sys.settrace(self.globaltrace)
    self.__run_backup()
    self.run = self.__run_backup
 
  def globaltrace(self, frame, event, arg):
    if event == 'call':
      return self.localtrace
    else:
      return None
 
  def localtrace(self, frame, event, arg):
    if self.killed:
      if event == 'line':
        raise SystemExit()
    return self.localtrace
 
  def kill(self):
    self.killed = True



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


def start_kernel():
    """starts the ipython kernel and returns the ipython app"""
    if sys._ipython_app and sys._ipython_kernel_running:
        return sys._ipython_app

    # The stdout/stderrs used by IPython. These get set after the kernel has started.
    ipy_stdout = sys.stdout
    ipy_stderr = sys.stderr


    # patch IPKernelApp.start so that it doesn't block
    def _IPKernelApp_start(self):
        nonlocal ipy_stdout, ipy_stderr

        if self.poller is not None:
            self.poller.start()
        self.kernel.start()

        # set up a timer to periodically poll the zmq ioloop
        self.loop = ioloop.IOLoop.current()
        self.aloop = asyncio.get_event_loop()

        _log.debug(f"_IPKernelApp_start(): self.kernel: {self.kernel}")
        _log.debug(f"_IPKernelApp_start(): self.loop: {self.loop}")
        _log.debug(f"_IPKernelApp_start(): self.aloop: {self.aloop}")

        #try:
        #    self.loop.start()
        #except KeyboardInterrupt:
        #    pass

        def kernel_loop():
            try:
                self.aloop.run_forever()
            except KeyboardInterrupt:
                pass

        self.kernel_thread = thread_with_trace(target=kernel_loop)
        self.kernel_thread.start()

        #def poll_ioloop():
        #    try:
        #        # Use the IPython stdout/stderr while running the kernel
        #        with PushStdout(ipy_stdout, ipy_stderr):
        #            # If the kernel has been closed then run the event loop until it gets to the
        #            # stop event added by IPKernelApp.shutdown_request
        #            if self.kernel.shell.exit_now:
        #                _log.debug("IPython kernel stopping (%s)" % self.connection_file)
        #                self.loop.start()
        #                sys._ipython_kernel_running = False
        #                return

        #            # otherwise call the event loop but stop immediately if there are no pending events
        #            self.loop.add_timeout(0, lambda: self.loop.add_callback(self.loop.stop))
        #            self.loop.start()
        #    except:
        #        _log.error("Error polling Jupyter loop", exc_info=True)

        #    #schedule_call(poll_ioloop, delay=0.1)
        #    threading.Timer(0.1, poll_ioloop).start()

        sys._ipython_kernel_running = True
        ##schedule_call(poll_ioloop, delay=0.1)
        #threading.Timer(0.1, poll_ioloop).start()

    IPKernelApp.start = _IPKernelApp_start

    # IPython expects sys.__stdout__ to be set, and keep the original values to
    # be used after IPython has set its own.
    sys.__stdout__ = sys_stdout = sys.stdout
    sys.__stderr__ = sys_stderr = sys.stderr

    # Get or create the IPKernelApp instance and set the 'connection_dir' property
    if IPKernelApp.initialized():
        ipy = IPKernelApp.instance()
    else:
        #ipy = IPKernelApp.instance(local_ns={})
        ipy = IPKernelApp.instance()
        ipy.initialize([])

    # call the API embed function, which will use the monkey-patched method above
    embed_kernel(local_ns={})

    # Keep a reference to the kernel even if this module is reloaded
    sys._ipython_app = ipy

    # Restore sys stdout/stderr and keep track of the IPython versions
    ipy_stdout = sys.stdout
    ipy_stderr = sys.stderr
    sys.stdout = sys_stdout
    sys.stderr = sys_stderr

    # patch user_global_ns so that it always references the user_ns dict
    setattr(ipy.shell.__class__, 'user_global_ns', property(lambda self: self.user_ns))

    # patch ipapp so anything else trying to get a terminal app (e.g. ipdb) gets our IPKernalApp.
    from IPython.terminal.ipapp import TerminalIPythonApp
    TerminalIPythonApp.instance = lambda: ipy

    _log.debug("start_kernel() finishing.")
    return ipy


