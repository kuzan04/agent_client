import os
import time
import base64
import binascii
import sys
import mysql.connector
from module import log, file, db

class startTask:
    def __init__(self, default, config):
        self._content = ["type", "status", "name", "host", "port", "detail", "cert"]
        self._path = default
        self._config = config
        self._list = os.listdir(config)
        self._start = True
        self.config = []
        self._token = ""

    def is_base64(self, s):
        try:
            base64.b64decode(s).decode("utf-8")
            return True
        except binascii.Error:
            return False

    def base64ToString(self, b):
        return base64.b64decode(b).decode('utf-8')

    def stringToBase64(self, plantText):
        return base64.b64encode(plantText)

    def _listC(self, f, detail):
        for i,j in zip(self._content, detail):
            f.write(f"{i}={j}\n")

    def _check(self):
        while "init.conf" not in self._list:
            try:
                token = input("Please enter the token you have: ")
                if self.is_base64(token) == True and token:
                    self._token = token
                    deatil = self.base64ToString(token).split("&&&")
                    if "AG1" in deatil or "AG2" in deatil or "AG3" in deatil or "AG4" in deatil:
                        f = open(f"{self._config}/init.conf", "w+")
                        self._listC(f, deatil)
                        f.close()
                        self._list = os.listdir(self._config)
                    else:
                        print("[Errno] Create init.conf not success, please check token incorrect.")
                elif self.is_base64(token) == False and token:
                    print("Token incorrect.\nPlease enter again or exit process.")
                else:
                    print("Bye.")
                    break
            except KeyboardInterrupt:
                print("\nBye.")
                break
            except Exception as e:
                print(str(e))
        return self._run()

    def setupConfig(self):
        try:
            f=open(os.path.join(self._config, "init.conf"), "r").readlines()
            for i in f:
                if i.find("#") == -1:
                    x=i.split("=")
                    self.config.append(x[1].strip("\n"))
        except Exception as e:
            print(str(e))
            sys.exit(1)
        finally:
            if len(self.config) < 6:
                print("[Errno] Please check init file.")
                sys.exit(1)

    def checkToken(self, fetch, i):
        if i == len(fetch):
            return -1
        elif self._token != fetch[i][-1]:
            return self.checkToken(fetch, (i+1))
        elif self._token == fetch[i][-1]:
            return fetch[i]

    def reverseToken(self, now):
        content = ""
        with open(f"{self._config}/init.conf", "r+") as lst:
            lines = lst.readlines()
            last = lines[-1]
            oldStatus = lines[1]
            for line in lines:
                if line is last and line.find("#") == -1 and line is not oldStatus:
                    x=line.split("=")
                    content+=x[1].strip("\n")
                elif line is not last and line.find('#') == -1 and line is not oldStatus:
                    x=line.split("=")
                    content+=x[1].strip("\n")+"&&&"
                elif line is not last and line.find('#') == -1 and now != 0 and line is oldStatus:
                    x=line.split("=")
                    content+=x[1].strip("\n")+"&&&"
                elif line is not last and line.find("#") == -1 and now == 0 and line is oldStatus:
                    content+=str(now)+"&&&"
            return self.stringToBase64(content.encode('utf-8')).decode('utf-8')

    def _update(self, old, now, i):
        if i == (len(old)+len(now))/2:
            return False
        elif i == 1 and old[i] == int(now[i]):
            return self._update(old, now, (i+1))
        elif i == 1 and old[i] != int(now[i]):
            return True
        elif i > 1 or i == 0 and old[i] != now[i]:
            return True
        elif i > 1 or i == 0 and old[i] == now[i]:
            return self._update(old, now, (i+1))

    def _updateFile(self, new, ip, port):
        new.insert(3, ip), new.insert(-1, port)
        if "AG1" in new or "AG2" in new or "AG3" in new or "AG4" in new:
            f=open(f"{self._config}/init.conf", "w+")
            self._listC(f, new)
            f.close()

    def _run(self):
        self.setupConfig()
        cnow = self.config
        ip, port = cnow.pop(3), cnow.pop(-2)
        while self._start:
            conn = mysql.connector.connect(
                host="127.0.0.1",
                user="root",
                password="P@ssw0rd",
                database="DOL_PDPA",
                auth_plugin="mysql_native_password"
            )
            cursor = conn.cursor()
            cursor.execute('SELECT pas.code, pam.agm_status, pam.agm_name, pam.config_detail, pam.agm_token FROM TB_TR_PDPA_AGENT_MANAGE as pam JOIN TB_TR_PDPA_AGENT_STORE as pas ON pam.ags_id = pas.ags_id;')
            commit = cursor.fetchall()
            if not self._token and int(self.config[1]) == 0:
                self._token = self.reverseToken(-1)
            elif not self._token and int(self.config[1]) == 1:
                self._token = self.reverseToken(0)
            else:
                rs = self.checkToken(commit, 0)
                if rs == -1:
                    print("[Errno] Client not match from manage.")
                    sys.exit(1)
                else:
                    rs = list(rs)
                    rs.pop()
                    if self._update(rs, cnow, 0) == False and rs[1] == 1:
                        print(-1)
                    elif self._update(rs, cnow, 0) == True and rs[1] == 0:
                        old = cnow
                        try:
                            self._updateFile(rs, ip, port)
                        except KeyboardInterrupt:
                            self._updateFile(old, ip, port)
                        finally:
                            self.config = []
                            self.setupConfig()
                    elif self._update(rs, cnow, 0) == True and rs[1] == 1:
                        self._updateFile(rs, ip, port)
                        self.config = []
                        self.setupConfig()
                        if self.config[0] == "AG1":
                            result = log.LogHash0(self.config[-1].split(","), self.config[0], self.config[2]).run()
                            return result
                        elif self.config[0] == "AG2":
                            result = file.dirFile(self.config[-1].split(","), self.config[0], self.config[2], self.config[3])
                            return result
                        elif self.config[0] == "AG3":
                            prepared = self.config[-1].split("&")
                            result = db.dbCheck(prepared[0], self.config[0], self.config[2], prepared[1], prepared[2], prepared[3], prepared[4], prepared[5:]).run()
                            return result
                        elif self.config[0] == "AG4":
                            prepared = self.config[-1].split(",")
                            return f"{self.config},{prepared}"
                        else:
                            print("[Errno] Type error.")
                    else:
                        pass
