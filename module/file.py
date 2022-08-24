import sys
import socket
import os
import time
from ftplib import FTP, error_perm
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

    def cleanhtml(self, raw_html):
        cleantext = re.sub(CLEANR, '', raw_html)
        return cleantext

    def ftpHandle(self, path, count, inform):
        _, _, filenames = next(walk(path), (None, None, []))
        for l in filenames:
            self.message.append(f"{self.code}#{self.name}|||{socket.gethostname()}|||{platform.system()}-{platform.release()}|||{path}|||{l}|||{inform}")
            time.sleep(2)
            ftp=FTP()
            ftp.connect(self._ip, self._port)
            ftp.login(self.user, self.passwd)
            ftp.cwd('files')
            ftp.retrlines('LIST')
            #if l.find('%20'):
            #    name_file = l.replace(' ','%')
            #    try:
            #        incent = urlopen(f"http://{config[-2]}/{inform}/{name_file}")
            #    except HTTPError as e:
            #        if e.code == 404:
            #            print('Err')
            #            nfl = open(f'{config[count]+l}', 'rb')
            #            ftp.cwd(inform)
            #            ftp.storbinary('STOR '+ name_file, nfl)
            #            #print('HTTPError: {}'.format(e.code))
            #    except URLError as e:
            #        print('URLError: {}'.format(e.reason))
            #else:
            #    # 200
            #    print("Ok")
            #    nfl = open(f'{config[count]+l}', 'rb')
            #    ftp.cwd(inform)
            #    ftp.storbinary('STOR ' + name_file, nfl)
            #    nfl.close()
            #    ftp.quit()

    def run(self):
        for i in self._path:
            self.ftpHandle()
        return self.message
            #count=1
            #while count > 0 and count < len(CONFIG)-5:
            #    conv_count = str(count)
            #    if len(conv_count) == 1:
            #        conv_count = "00"+conv_count
            #    elif len(conv_count) == 2:
            #        conv_count = "0"+conv_count
            #        inform = CONFIG[0]+'02'+conv_count+"@"+socket.gethostname()
            #        ftpHandle(CONFIG, count, inform)
            #        count+=1
