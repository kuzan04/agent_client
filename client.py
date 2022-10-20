#======================================================
# Package at used
#======================================================
import sys
import socket
import os
import shutil
import time
import datetime
import re
import mysql.connector
import base64
import binascii
from module import cert, mode, connect, sniffer
#======================================================
# Convert Base64 to Text
#======================================================
def is_base64(s):
    try:
        base64.b64decode(s).decode("utf-8")
        return True
    except binascii.Error:
        return False
#======================================================
def base64ToString(b):
    return base64.b64decode(b).decode('utf-8')
#======================================================
# Write config file
#======================================================
def write(f, h, d):
    for i,j in zip(h, d):
        f.write(f"{i}={j}\n")
#======================================================
# Checking config file
#======================================================
def check(_l, _c):
    while "init.conf" not in _l:
        try:
            token = input("Please enter the token you have: ")
            if is_base64(token) == True and token:
                c_token = token
                deatil = base64ToString(token).split("&&&")
                if "AG1" in deatil or "AG2" in deatil or "AG3" in deatil or "AG4" in deatil:
                    f = open(f"{_c}/init.conf", "w+")
                    write(f, ["type", "status", "name", "host", "port", "detail", "cert"], deatil)
                    f.close()
                    return c_token, os.listdir(_c)
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
#======================================================
# Setup config file
#======================================================
def setup(_c):
    c = []
    try:
        f=open(os.path.join(_c, "init.conf"), "r").readlines()
        for i in f:
            if i.find("#") == -1:
                x=i.split("=")
                c.append(x[1].strip("\n"))
            else:
                pass
    except Exception as e:
        print(str(e))
        sys.exit(1)
    finally:
        if len(c) < 6:
            print("[Errno] Please check init file.")
            sys.exit(1)
        else:
            return c
#======================================================
# Main
#======================================================
if __name__ == "__main__":
    __location__ = os.path.realpath(os.path.join(os.getcwd(), os.path.dirname(__file__)))
    __config__ = ""
    __ssl__ = []
    if "config" in os.listdir(__location__):
        __config__ = os.path.join(__location__, "config")
        if "ssl" in os.listdir(__config__):
            for i in os.listdir(os.path.join(__config__, "ssl")):
                __ssl__.append(os.path.join(__config__, f"ssl/{i}"))
    else:
        print("[Errno] Please check directory config missing.")
        sys.exit(1)
    # Connection Main Database.
    conn = mysql.connector.connect( host="127.0.0.1", user="root", password="P@ssw0rd", auth_plugin="mysql_native_password", database="DOL_PDPA_LOCAL" )
    # Call fn check.
    __token__, c_list = check(os.listdir(__config__), __config__)
    if c_token and c_list:
        __init__ = setup(c_list)
        db = None
        if config[0] == "AG3":
            detail = config[-1].split("&")
            if detail[0] == 1: # MySQL.
                db = mysql.connector.connect( host=detail[1], user=detail[2], password=detail[3], auth_plugin="mysql_native_password", database=detail[4] )
            elif detail[0] == 0: # Hold for oracle database.
                pass
            else: # Hold for another service database.
                pass
        else:
            db = None
        while True:
            process = mode.startTask(__config__, __init__, __token__, __ssl__, conn, db).run()
            if isinstance(process, str):
                config, prepared = eval(process)[0], eval(process)[1]
                if int(prepared[0]) == 0: # FTP
                    sniffer.taskSnif(config, prepared[0], 21, prepared[-1], __config__).run()
                elif int(prepared[0]) == 1: # Syslog
                    sniffer.taskSnif(config, prepared[0], 514, prepared[-1], __config__).run()
                else:
                    pass
            else:
                pass
    else:
        pass
