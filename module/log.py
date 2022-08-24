import sys
import socket
import time
import hashlib
import platform
from os import walk
from os import listdir
from os.path import isfile, join
from datetime import datetime

class LogHash0:
    def __init__(self, path, code, name):
        self._path = path
        self.code = code
        self.name = name
        self.message = []

    def sha256sum(filename):
        h = hashlib.sha256()
        b = bytearray(128*1024)
        mv = memoryview(b)
        try:
            with open(filename, 'rb', buffering=0) as f:
                for n in iter(lambda : f.readinto(mv), 0):
                    h.update(mv[:n])
                    return h.hexdigest()
        except Exception as e:
            print(str(e))
            sys.exit(1)

    def checkLog(self, _path):
        _, _, filenames = next(walk(_path), (None, None, []))
        for l in filenames:
            print(l)
            fl = open(f"{_path+l}",'rb')
            le = len(fl.readlines())
            print(le)
            time.sleep(2)
            fl1 = open(f"{_path+l}",'rb')
            nle = len(fl1.readlines())
            print(nle)
            if nle > le:
                h = self.sha256sum(f"{_path+l}")
                self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()} {platform.release()}|||{_path}|||{l}|||{str(nle)}|||{h}")
            else:
                self.message.append(None)

    def run(self):
        for i in self._path:
            self.checkLog(i)
        return self.message
