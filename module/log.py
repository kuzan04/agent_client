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

    def sha256sum(self, filename):
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

    def md5sum(self, filename):
        h = hashlib.md5()
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

    def sha1sum(self, filename):
        h = hashlib.sha1()
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
            fl = open(f"{_path+l}",'rb')
            le = len(fl.readlines())
            fl.close()
            time.sleep(5)
            fl1 = open(f"{_path+l}",'rb')
            nle = len(fl1.readlines())
            fl1.close()
            if nle > le:
                sha256 = self.sha256sum(f"{_path+l}")
                md5 = self.md5sum(f"{_path+l}")
                sha1 = self.sha1sum(f"{_path+l}")
                self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()} {platform.release()}|||{_path}|||{l}|||{str(nle)}|||{sha256}|||{md5}|||{sha1}")
            else:
                self.message.append(None)

    def run(self):
        for i in self._path:
            self.checkLog(i)
        return self.message
