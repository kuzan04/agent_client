import sys
import socket
import subprocess as sub
import datetime
import ftplib
import os
from pathlib import Path
import logging as log
import logging.handlers as logHandle
from itertools import chain


class SizeFile:
    def __init__(self, n, s, d):
        self.name = n
        self.source = s
        self.destination = d

class taskSnif:
    def __init__(self, path, conn, init, token, host, timeout=120, user="ftpuser", passwd="ftpuser"):
        self._config = path
        self._conn = conn
        self._host = host
        self._token = token
        self.config = init[:-1]
        self.detail = init[-1].split(",")
        self._port = 21 if int(init[-1].split(",")[0]) == 0 else 514
        # or same a > b and 21 or 514
        self._start = True
        self._format = 'utf-8'
        self._time = timeout
        self.username = user
        self.password = passwd
        self.name = None
        self.time = None

    def checkToken(self, fetch, i):
        if i == len(fetch):
            return -1
        elif self._token != fetch[i][-1]:
            return self.checkToken(fetch, (i+1))
        elif self._token == fetch[i][-1]:
            return fetch[i]

    def _update(self, now, i):
        if i == (len(self.config)+len(now))/2:
            return False
        elif i == 1 and self.config[i] == int(now[i]):
            return self._update(now, (i+1))
        elif i == 1 and self.config[i] != int(now[i]):
            return True
        elif i > 1 or i == 0 and self.config[i] != now[i]:
            return True
        elif i > 1 or i == 0 and self.config[i] == now[i]:
            return self._update(now, (i+1))

    def _updateFile(self, new):
        if "AG1" in new or "AG2" in new or "AG3" in new or "AG4" in new:
            with open(f"{self._config}/init.conf", "w+") as f:
                for i, j in zip(["type", "status", "name", "host", "port", "detail", "tk"], new):
                    f.write(f"{i}:={j}\n")
                f.close()

    def checkList(self, a0, a1, i, j):
        if i == len(a0):
            return a0[(i-1)]
        elif j == len(a1):
            return self.checkList(a0, a1, (i+1), j)
        elif a0[i] != a1[j]:
            return self.checkList(a0, a1, i, (j+1))
        elif a0[i] == a1[j]:
            return 0

    def checkArray(self, a0, a1, i, j):
        if i == len(a0):
            return -1
        elif j == len(a1):
            return self.checkArray(a0, a1, (i+1), j)
        elif a0[i] != a1[j]:
            return self.checkArray(a0, a1, i, (j+1))
        elif a0[i] == a1[j]:
            return j

    def find_tuple(self, _tuple, mark, c, i):
        if i == len(_tuple):
            return -1
        elif _tuple[i][1] == mark and _tuple[i][-1] == c:
            return _tuple[i]
        else:
            return self.find_tuple(_tuple, mark, c, (i+1))

    def find_tuple1(self, _t, a, i):
        if i == len(a):
            return -1
        elif list(_t)[0] == a[i]:
            return i
        else:
            return self.find_tuple1(_t, a, (i+1))

    def findLength(self, _l, ftp, s):
        if _l[-1] == s:
            sub0 = os.path.getsize(os.path.join(self.detail[-1], _l[-1]))
            # Switch to Binary
            ftp.sendcmd('TYPE I')
            sub1 = ftp.size(s)
            return SizeFile(_l[-1], sub0, sub1)
        else:
            return 0

    def checkLength(self, o):
        if o.source == o.destination:
            return 0
        elif o.source != o.destination:
            return 1

    def convertNoneType(self, f):
        l=[]
        for i in f:
            i = i.split(' ')
            if i[-1].find('.DS_Store'):
                l.append(i[-1])
        return l

    def createDirectory(self, files, i):
        if i == len(files):
            return 0
        elif files[i] != self._host:
            return self.createDirectory(files, (i+1))
        elif files[i] == self._host:
            return 1

    def convertSetToList(self, f, rs=None):
        if rs is None:
            rs = []
        for n in f:
            rs.append(n)
        return rs

    def successFile(self):
        iter_files = sorted(Path(self.detail[-1]).iterdir(), key=os.path.getmtime)
        files = [i.name for i in iter_files]
        if len(files) >  1 and '.DS_Store' in files:
            files.remove('.DS_Store')
            if os.path.exists(os.path.join(self.detail[-1], files[0])):
                os.remove(os.path.join(self.detail[-1], files[0]))
            else:
                return -1
        elif len(files) > 1 and '.DS_Store' not in files:
            if os.path.exists(os.path.join(self.detail[-1], files[0])):
                os.remove(os.path.join(self.detail[-1], files[0]))
            else:
                return -1

    def writeSniff(self, b):
        date = str(datetime.datetime.now()).replace(" ", ",").split(',')
        fulltime = date[-1].split('.')
        # date.pop(),
        fulltime.pop()
        hour = int(fulltime[0].split(":")[0])
        # Check minutes => int(time[0].split(":")[-2])
        # Check hours => int(time[0].split(":")[0])
        if hour < 24 and hour == self.time:
            with open(str(self.name), 'a+') as n:
                n.write(b.decode(self._format)+"\n")
                n.close()
        else:
            self.successFile()
            self.name = os.path.join(self.detail[-1], f"{self._host},{hour}:00,{date[-1]}.snf")
            with open(str(self.name), 'a+') as n:
                n.write(b.decode(self._format)+"\n")
                n.close()
            self.time = hour

    # Mode 0
    def sendFTP(self, c, r):
        try:
            ftp = ftplib.FTP_TLS()
            ftp.connect(self.config[3], int(self._port), self._time)
            ftp.sendcmd(f'USER {self.username}')
            ftp.sendcmd(f'PASS {self.password}')
            ftp_directory = []
            ftp.dir(ftp_directory.append)
            ftp_directory = self.convertNoneType(ftp_directory)
            local_files = os.listdir(self.detail[-1])
            if '.DS_Store' in local_files:
                local_files.remove('.DS_Store')
            else:
                pass
            if self.createDirectory(ftp_directory, 0) == 0:
                ftp.mkd(self._host)
                ftp.cwd(self._host)
                ftp_files = self.convertSetToList(ftp.nlst())
                if len(ftp_files) == 0 and self._host in local_files:
                    with open(os.path.join(self.detail[-1], local_files[local_files.index(self._host)]), 'rb') as f:
                        dest_path = f'/{self._host}/{local_files[local_files.index(self._host)]}'
                        ftp.storbinary(f'STOR {dest_path}', f)
                        f.close()
                else:
                    pass
            elif self.createDirectory(ftp_directory, 0) == 1:
                ftp.cwd(self._host)
                ftp_files = self.convertSetToList(ftp.nlst())
                rs = self.checkList(local_files, ftp_files, 0, 0)
                if rs != 0:
                    with open(os.path.join(self.detail[-1], str(rs)), 'rb') as f:
                        dest_path = f'/{self._host}/{rs}'
                        ftp.storbinary(f'STOR {dest_path}', f)
                        f.close()
                else:
                    rs1 = self.findLength(local_files, ftp, ftp_files[self.checkArray(local_files, ftp_files, 0, 0)])
                    # Switch backto ascii
                    ftp.sendcmd('TYPE A')
                    if isinstance(rs1, SizeFile):
                        if self.checkLength(rs1) == 1:
                            with open(os.path.join(self.detail[-1], local_files[-1]), 'rb') as f:
                                dest_path=f'/{self._host}/{local_files[-1]}'
                                ftp.storbinary(f'STOR {dest_path}', f)
                                f.close()
                        else:
                            pass
            # Exit FTP
            ftp.quit()
            # Manage history client
            select = ()
            select = self.find_tuple(r, self.config[2], "AG4", 0)
            c.execute('SELECT agm_id FROM TB_TR_PDPA_AGENT_LISTEN_HISTORY GROUP BY agm_id;')
            find_history = [list(x) for x in c.fetchall()]
            find_history = list(chain.from_iterable(find_history))
            idx = self.find_tuple1(select, find_history, 0)
            if idx == -1:
                c.execute(f"INSERT INTO TB_TR_PDPA_AGENT_LISTEN_HISTORY (agm_id) VALUE ({select[0]})")
                self._conn.commit()
            else:
                c.execute(f"UPDATE TB_TR_PDPA_AGENT_LISTEN_HISTORY SET _get_ = NOW() WHERE agm_id = {find_history[idx]}")
                self._conn.commit()
        except Exception as e:
            err=str(e).split(" ")
            if err[0] == 'timed' or err[1] == 'out':
                print(str(e))
                print('[Errno] sendFTP() Please check file config.')
                sys.exit(1)
            else:
                err=err[1].replace("]","")
                try:
                    if int(err) != 49:
                        print(str(e))
                        print('[Errno] sendFTP() Please check file config.')
                        sys.exit(1)
                except Exception as e:
                    print(str(e))
                    print('[Errno] sendFTP() Please check FTP server.')
                    sys.exit(1)

    # Mode 1
    def sendSyslog(self, m, c, r):
        try:
            # History client
            logger = log.getLogger()
            logger.setLevel(log.INFO) # CRITICAL = 50, ERROR = 40, WARNING = 30, INFO = 20, DEBUG = 10, NOTSET = 0 **NOTE** handler syslog server ip can't sure dynamic must manually.
            handler = logHandle.SysLogHandler(address = (self.config[3], int(self._port)), socktype=socket.SOCK_DGRAM)
            logger.addHandler(handler)
            logger.info(m)
            logger.removeHandler(handler)
            handler.close()
            log.shutdown()
            # Manage history client
            select = self.find_tuple(r, self.config[2], "AG4", 0)
            c.execute('SELECT agm_id FROM TB_TR_PDPA_AGENT_LISTEN_HISTORY GROUP BY agm_id;')
            find_history = [list(x) for x in c.fetchall()]
            find_history = list(chain.from_iterable(find_history))
            idx = self.find_tuple1(select, find_history, 0)
            if idx == -1:
                c.execute(f"INSERT INTO TB_TR_PDPA_AGENT_LISTEN_HISTORY (agm_id) VALUE ({select[0]})")
                self._conn.commit()
            else:
                c.execute(f"UPDATE TB_TR_PDPA_AGENT_LISTEN_HISTORY SET _get_ = NOW() WHERE agm_id = {find_history[idx]}")
                self._conn.commit()
        except Exception as e:
            print(e)
            sys.exit(1)

    # Main process.
    def tcpDump(self, p):
        cursor = self._conn.cursor()
        cursor.execute("SELECT pam.agm_id, pam.agm_name, pas.code FROM TB_TR_PDPA_AGENT_MANAGE as pam JOIN TB_TR_PDPA_AGENT_STORE as pas ON pam.ags_id = pas.ags_id;")
        res = cursor.fetchall()
        self._conn.commit()
        for row in iter(p.stdout.readline, b''):
            # Sub process.
            if int(self.detail[0]) == 0:
                self.writeSniff(row.rstrip())
                self.sendFTP(cursor, res)
            elif int(self.detail[0]) == 1:
                # Sub process.
                self.sendSyslog(row.rstrip(), cursor, res)

    def run(self):
        while self._start:
            cursor = self._conn.cursor()
            cursor.execute('SELECT pas.code, pam.agm_status, pam.agm_name, pam.config_detail, pam.agm_token FROM TB_TR_PDPA_AGENT_MANAGE as pam JOIN TB_TR_PDPA_AGENT_STORE as pas ON pam.ags_id = pas.ags_id;')
            commit = cursor.fetchall()
            self._conn.commit()
            rs = self.checkToken(commit, 0)
            if rs == -1:
                print("[Errno] Client not match from manage.")
                sys.exit(1)
            else:
                rs = list(rs)
                rs.insert(3, self.config[3]), rs.insert(4, self.config[4])
                if self._update(rs, 0) == True and int(rs[1]) == 1 and int(self.config[1]) == 1:
                    self._start = False
                else:
                    try:
                        self._updateFile(rs)
                    except KeyboardInterrupt:
                        self._updateFile(self.config)
                    finally:
                        self.config = []
                        try:
                            f=open(os.path.join(self._config, "init.conf"), "r").readlines()
                            for i in f:
                                if i.find('#') == -1:
                                    x=i.split(":=")
                                    self.config.append(x[1].strip("\n"))
                                else:
                                    pass
                        except Exception as e:
                            print(str(e))
                            sys.exit(1)
                        finally:
                            if len(self.config) < 7:
                                print("[Errno] Please check init file.")
                                sys.exit(1)
                            else:
                                pass
        else:
            try:
                if int(self.detail[0]) == 0 or int(self.detail[0]) == 1:
                    process = sub.Popen(('sudo', 'tcpdump', '-l'), stdout=sub.PIPE)
                    self.tcpDump(process)
                else:
                    print("[Errno] OS not support, Please check informant on https://alltra.com or contact develop alltra@gmail.com")
                    sys.exit(1)
            except Exception as e:
                print(str(e))
                sys.exit(1)
            except KeyboardInterrupt:
                print("\nCaught keyboard interrupt, exiting")
                sys.exit(1)
