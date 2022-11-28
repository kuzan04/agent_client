import os
import time
import sys
import mysql.connector
from module import log, file, db, connect

class startTask:
    def __init__(self, path, init, token, ssl, conn, db):
        self._config = path
        self._start = True
        self._token = token
        self._ssl = ssl
        self._store = {}
        self._conn = conn
        self._select = db
        self.config = init

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
            f=open(f"{self._config}/init.conf", "w+")
            for i,j in zip(["type", "status", "name", "host", "port", "detail", "tk"], new):
                f.write(f"{i}:={j}\n")
            f.close()

    def _run(self):
        while self._start:
            cursor = self._conn.cursor()
            cursor.execute('SELECT pas.code, pam.agm_status, pam.agm_name, pam.config_detail, pam.agm_token FROM TB_TR_PDPA_AGENT_MANAGE as pam JOIN TB_TR_PDPA_AGENT_STORE as pas ON pam.ags_id = pas.ags_id;')
            commit = cursor.fetchall()
            self._conn.commit()
            self._start = False
        else:
            rs = self.checkToken(commit, 0)
            if rs == -1:
                print("[Errno] Client not match from manage.")
                sys.exit(1)
            else:
                rs = list(rs)
                rs.insert(3, self.config[2]), rs.insert(4, self.config[3])
                if self._update(rs, 0) == True and rs[1] == 1:
                    if self.config[0] == "AG1":
                        cursor.execute('SELECT path, name_file FROM TB_TR_PDPA_AGENT_LOG0_HASH;')
                        backup = cursor.fetchall()
                        result, store = log.LogHash0(self.config[-1].split(","), self.config[0], self.config[2], self._store, backup).run()
                        self._store = store
                        self._connect(result, "AG1")
                        self._start = True
                    elif self.config[0] == "AG2":
                        result = file.dirFile(self.config[-1].split(","), self.config[0], self.config[2], self.config[3]).run()
                        self._connect(result, "AG2")
                        self._start = True
                    elif self.config[0] == "AG3":
                        prepared = self.config[-2].split(":")
                        result = db.dbCheck(prepared[0], self.config[0], self.config[2], self._select, prepared[5:]).run()
                        self._connect(result, "AG3")
                        self._start = True
                    else:
                        print("[Errno] Type error.")
                else:
                    try:
                        self._updateFile(rs)
                    except KeyboardInterrupt:
                        self._updateFile(self.config)
                    finally:
                        self.config=[]
                        try:
                            f=open(os.path.join(self._config, "init.conf"), "r").readlines()
                            for i in f:
                                if i.find("#") == -1:
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

    def _connect(self, msg, _type):
        client_cert = [x for x in self._ssl if ".crt" in x.split("/")[-1]].pop()
        client_key = [x for x in self._ssl if ".key" in x.split("/")[-1]].pop()
        c = connect.SSLClient( self.config[3], int(self.config[-2]), client_cert, client_key )
        c.connect()
        for i in msg:
            if i is not None:
                c.send(i)
            else:
                pass
        c.close()
