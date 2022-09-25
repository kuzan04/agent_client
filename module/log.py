import sys
import socket
import time
import hashlib
import platform
import os
from os import walk
from os import listdir
from os.path import isfile, join
from datetime import datetime

class LogHash0:
    def __init__(self, path, code, name, store, backup):
        self._path = path
        self.code = code
        self.name = name
        self.message = []
        self._store = store
        self._backup = backup

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

    def fileMatch(self, old, new, size, i):
        try:
            _old = list(old[i])
            if i == size:
                return -1
            elif _old[0] == new[0] and _old[1] == new[1]:
                if len(self._store) == 0:
                    self._store[_old[0]] = []
                    self._store[_old[0]].append(_old[1])
                else:
                    self._store[_old[0]].append(_old[1])
                return True
            else:
                return self.fileMatch(old, new, size, (i+1))
        except IndexError:
            return False

    def checkLog(self, _path):
        if len(os.listdir(_path)) == 0:
            time.sleep(5)
            if len(os.listdir(_path)) > 0:
                _, _, filename = next(walk(_path), (None, None, []))
                contents_len = len(open(f"{_path+filename[0]}", "rb").readlines())
                sha256 = self.sha256sum(f"{_path+filename[0]}")
                md5 = self.md5sum(f"{_path+filename[0]}")
                sha1 = self.sha1sum(f"{_path+filename[0]}")
                self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()} {platform.release()}|||{_path}|||{filename[0]}|||{str(contents_len)}|||{sha256}|||{md5}|||{sha1}")
                self._store[_path] = []
                self._store[_path].append(filename)
            else:
                pass
        else:
            _, _, filenames = next(walk(_path), (None, None, []))
            for l in filenames:
                try:
                    if l in self._store[_path]:
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
                    else:
                        contents_len = len(open(f"{_path+l}", "rb").readlines())
                        sha256 = self.sha256sum(f"{_path+l}")
                        md5 = self.md5sum(f"{_path+l}")
                        sha1 = self.sha1sum(f"{_path+l}")
                        self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()} {platform.release()}|||{_path}|||{l}|||{str(contents_len)}|||{sha256}|||{md5}|||{sha1}")
                        self._store[_path].append(l)
                except KeyError:
                    check = self.fileMatch(self._backup, [_path, l], len(filenames), 0)
                    if check == True:
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
                    else:
                        contents_len = len(open(f"{_path+l}", "rb").readlines())
                        sha256 = self.sha256sum(f"{_path+l}")
                        md5 = self.md5sum(f"{_path+l}")
                        sha1 = self.sha1sum(f"{_path+l}")
                        self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()} {platform.release()}|||{_path}|||{l}|||{str(contents_len)}|||{sha256}|||{md5}|||{sha1}")
                        self._store[_path].append(l)

    def run(self):
        for i in self._path:
            self.checkLog(i)
        return self.message, self._store
