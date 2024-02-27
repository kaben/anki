import asyncio
import logging
import os
import sys
import threading

import flask
import PyQt6
from ipykernel.embed import embed_kernel
from ipykernel.kernelapp import IPKernelApp
from PyQt6.QtCore import *

# conflicting Qt and qFuzzyCompare definitions require an ignore
from PyQt6.QtGui import *  # type: ignore[misc,assignment]
from PyQt6.QtNetwork import QLocalServer, QLocalSocket, QNetworkProxy
from PyQt6.QtWebChannel import QWebChannel
from PyQt6.QtWebEngineCore import *
from PyQt6.QtWebEngineWidgets import *
from PyQt6.QtWidgets import *
from waitress.server import create_server

flask_app = flask.Flask(__name__.split('.')[0])

@flask_app.route("/")
def hello_world():
  return "<p>Hello, World!</p>"

@flask_app.route("/start_jupyter_kernel", methods=['GET', 'POST'])
def start_jupyter_kernel():
  print(flask.request)
  return f"<p>{flask.request}</p>"


class FlaskAppWorker(threading.Thread):
  _ready = threading.Event()
  daemon = True

  def __init__(self) -> None:
    super().__init__()
    self.is_shutdown = False

  def run(self):
    try:
      # idempotent if logging has already been set up
      logging.basicConfig()
      logging.getLogger("waitress").setLevel(logging.ERROR)

      desired_host = "127.0.0.1"
      desired_port = 5000
      self.server = create_server(
        flask_app,
        host=desired_host,
        port=desired_port,
        clear_untrusted_proxy_headers=True,
      )
      print(
        "Serving on http://%s:%s"
        % (self.server.effective_host, self.server.effective_port)  # type: ignore
      )
      self._ready.set()
      self.server.run()

    except Exception:
      if not self.is_shutdown:
        raise

  def shutdown(self) -> None:
    self.is_shutdown = True
    sockets = list(self.server._map.values())  # type: ignore
    for socket in sockets:
      socket.handle_close()
    # https://github.com/Pylons/webtest/blob/4b8a3ebf984185ff4fefb31b4d0cf82682e1fcf7/webtest/http.py#L93-L104
    self.server.task_dispatcher.shutdown()


class JupyterKernelWorker(threading.Thread):
  def run(self):
    self.loop = asyncio.new_event_loop()
    asyncio.set_event_loop(self.loop)
    ipy_kernel_app = IPKernelApp()


class MainWindow(QMainWindow):
  def __init__(self):
    super().__init__()
    self.worker = FlaskAppWorker()
    self.worker.start()
    layout = QVBoxLayout()
    button = QPushButton('hello, world')
    layout.addWidget(button)
    widget = QWidget()
    widget.setLayout(layout)
    self.setCentralWidget(widget)

  def shutdown(self):
    print("shutting down.")
    self.worker.shutdown()
    self.worker.join()
    
if __name__ == "__main__":
  app = QApplication(sys.argv)
  window = MainWindow()
  window.show()
  app.aboutToQuit.connect(window.shutdown)
  sys.exit(app.exec())