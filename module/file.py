import socket
import os
import ftplib
from os import walk
import platform
#import pysftp


class dirFile:
    def __init__(self, path, code, name, ip, port=21, username="ftpuser", password="ftpuser", own=None, group=None):
        self._path = path
        self.code = code
        self.name = name
        self._ip = ip
        self._port = port
        self.user = username
        self.passwd = password
        self._own = own
        self._group = group
        self.message = []

    def convertNoneType(self, _l):
        re = []
        for i in _l:
            i = i.split(" ")
            if ".DS_Store" not in i[-1]:
                re.append(i[-1])
        return re

    def ftpHandle(self, path, inform):
        _, _, filenames = next(walk(path), (None, None, []))
        filenames = [x for x in filenames if x.endswith('.log') or x.endswith('.xls') or x.endswith('.xlsx') or x.endswith('csv') or x.endswith('.evtx')]
        for l in filenames:
            if l != ".DS_Store":
                name_file = f"{inform}{l}"
                content_file = open(os.path.join(path, l), 'rb')
                size = os.path.getsize(os.path.join(path, l))
                if str(self._port)[-2:] == "21":
                    ftp=ftplib.FTP_TLS()
                    ftp.connect(self._ip, self._port)
                    ftp.sendcmd(f"USER {self.user}")
                    ftp.sendcmd(f"PASS {self.passwd}")
                    ftp.storbinary(f'STOR {name_file}', content_file)
                    content_file.close()
                    ftp.quit()
                    self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()}-{platform.release()}|||{path}|||{l}|||{size}|||{inform}")
                elif str(self._port)[-2:] == "22":
                    pass
                    '''with pysftp.Connection(self._ip, username=self.user, password=self.passwd) as sftp:
                        sftp.cd('/')
                        ftp.put(os.path.join(path, l), os.path.join(path, name_file))
                        sftp.close()
                        self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()}-{platform.release()}|||{path}|||{l}|||{size}|||{inform}")'''
                else:
                    pass
            else:
                pass

    def run(self):
        for i in range(len(self._path)):
            inform = ""
            if len(str(i)) == 1:
                inform = f"{self.code}@00{i+1}@{self.name}@"
            elif len(str(i)) == 2:
                inform = f"{self.code}@0{i+1}@{self.name}@"
            self.ftpHandle(self._path[i], inform)
        return self.message
