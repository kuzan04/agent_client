from sys import platform
import socket
import ssl

class SSLClient:
    def __init__ (self, server_host, server_port, client_cert, client_key):
        self.server_host = server_host
        self.server_port = int(server_port)
        #self.sni_hostname = sni_hostname
        self._context = ssl.SSLContext()
        self._context.load_cert_chain(client_cert, client_key)
        self._sock = None
        self._ssock = None

    def __del__(self):
        self.close()

    def connect(self):
        if platform == "darwin" and platform == "linux" and platform != "win32":
            self._sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        else:
            self._sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._ssock = self._context.wrap_socket(self._sock,)
        self._ssock.connect((self.server_host, self.server_port))

    def send(self, msg):
        self._ssock.send(msg.encode())

    def close(self):
        self._ssock.close()
