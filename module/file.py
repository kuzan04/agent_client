import sys
import socket
import os
import time
import pysftp
import ftplib
from os import walk
from os import listdir
from os.path import isfile, join
#from urllib.request import urlopen
#from urllib.error import URLError, HTTPError
#from urllib.parse import quote
import platform
import re

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

    def convertNoneType(self, l):
        re = []
        for i in l:
            i = i.split(" ")
            if ".DS_Store" not in i[-1]:
                re.append(i[-1])
        return re

    # Option
    def cleanhtml(self, raw_html):
        cleantext = re.sub(CLEANR, '', raw_html)
        return cleantext

    def ftpHandle(self, path, count, inform):
        _, _, filenames = next(walk(path), (None, None, []))
        for l in filenames:
            if l != ".DS_Store":
                name_file = f"{inform}{l}"
                content_file = open(os.path.join(path, l), 'rb')
                size = os.path.getsize(os.path.join(path, l))
                if str(self._port)[-2:] == "21":
                    ftp=ftplib.FTP_TLS()
                    ftp.connect(self._ip, self._port)
                    ftp.login(self.user, self.passwd)
                    ftp.prot_p()
                    ftp.cwd('/')
                    #all_file_ftp = []
                    #ftp.dir(all_file_ftp.append)
                    #all_file_ftp = self.convertNoneType(all_file_ftp)
                    ftp.storbinary('STOR '+ name_file, content_file)
                    content_file.close()
                    ftp.quit()
                    self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()}-{platform.release()}|||{path}|||{l}|||{size}|||{inform}")
                elif str(self._port)[-2:] == "22":
                    with pysftp.Connection(self._ip, username=self.user, password=self.passwd) as sftp:
                        sftp.cd('/')
                        ftp.put(os.path.join(path, l), os.path.join(path, name_file))
                        sftp.close()
                        self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()}-{platform.release()}|||{path}|||{l}|||{size}|||{inform}")
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
            self.ftpHandle(self._path[i], i, inform)
        return self.message
