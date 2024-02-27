import asyncio
import sys
import threading

import jupyter_server.serverapp
import jupyterlab.labapp
import PyQt6
from PyQt6.QtCore import *

# conflicting Qt and qFuzzyCompare definitions require an ignore
from PyQt6.QtGui import *  # type: ignore[misc,assignment]
from PyQt6.QtNetwork import QLocalServer, QLocalSocket, QNetworkProxy
from PyQt6.QtWebChannel import QWebChannel
from PyQt6.QtWebEngineCore import *
from PyQt6.QtWebEngineWidgets import *
from PyQt6.QtWidgets import *
from traitlets.config import Config
from traitlets.config.application import boolean_flag, catch_config_error


class MyServerApp(jupyter_server.serverapp.ServerApp):
  @catch_config_error
  def initialize(
    self,
    argv=None,
    find_extensions=True,
    new_httpserver=True,
    starter_extension=None,
  ):
    print("MyServerApp.initialize()...")
    self._init_asyncio_patch()
    # Parse command line, load ServerApp config files,
    # and update ServerApp config.
    #super().initialize(argv=argv)
    jupyterlab.labapp.JupyterApp.initialize(self, argv=argv)
    if self._dispatching:
      return
    # initialize io loop as early as possible,
    # so configurables, extensions may reference the event loop
    self.init_ioloop()

    # Then, use extensions' config loading mechanism to
    # update config. ServerApp config takes precedence.
    if find_extensions:
      self.find_server_extensions()
    self.init_logging()
    self.init_event_logger()
    self.init_server_extensions()

    # Special case the starter extension and load
    # any server configuration is provides.
    if starter_extension:
      # Configure ServerApp based on named extension.
      point = self.extension_manager.extension_points[starter_extension]
      # Set starter_app property.
      if point.app:
        self._starter_app = point.app
      # Load any configuration that comes from the Extension point.
      self.update_config(Config(point.config))

    # Initialize other pieces of the server.
    self.init_resources()
    self.init_configurables()
    self.init_components()
    self.init_webapp()
    #self.init_signal()
    self.load_server_extensions()
    self.init_mime_overrides()
    self.init_shutdown_no_activity()
    if new_httpserver:
      self.init_httpserver()


class MyExtensionApp(jupyterlab.labapp.LabApp):
  serverapp_class = MyServerApp

  @classmethod
  def make_serverapp(cls, **kwargs):
    print("MyExtensionApp.make_serverapp()...")
    return cls.serverapp_class.instance(**kwargs)

  @classmethod
  def initialize_server(cls, argv=None, load_other_extensions=True, **kwargs):
    print("MyExtensionApp.initialize_server()...")
    jpserver_extensions = {cls.get_extension_package(): True}
    find_extensions = cls.load_other_extensions
    if "jpserver_extensions" in cls.serverapp_config:
      jpserver_extensions.update(cls.serverapp_config["jpserver_extensions"])
      cls.serverapp_config["jpserver_extensions"] = jpserver_extensions
      find_extensions = False
    serverapp = cls.make_serverapp(jpserver_extensions=jpserver_extensions, **kwargs)
    serverapp.aliases.update(cls.aliases)
    serverapp.initialize(
      argv=argv or [],
      starter_extension=cls.name,
      find_extensions=find_extensions,
    )
    return serverapp


class Worker(threading.Thread):
  def run(self):
    #asyncio.run(self.main())
    self.loop = asyncio.new_event_loop()
    asyncio.set_event_loop(self.loop)
    self.serverapp = MyExtensionApp.initialize_server(sys.argv)
    self.serverapp.start()

class MainWindow(QMainWindow):
  def __init__(self):
    super().__init__()
    self.worker = Worker()
    self.worker.start()

  def shutdown(self):
    print("shutting down.")
    self.worker.serverapp.stop()
    self.worker.join()
    
if __name__ == "__main__":
  app = QApplication(sys.argv)
  window = MainWindow()
  window.show()
  app.aboutToQuit.connect(window.shutdown)
  sys.exit(app.exec())