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
        f.write(f"{i}:={j}\n")
#======================================================
# Checking config file
#======================================================
def check(_l, _c):
    while "init.conf" not in _l:
        try:
            token = input("Please enter the token you have: ")
            if is_base64(token) == True and token:
                detail = base64ToString(token).split("&&&")
                detail.append(token)
                if "AG1" in detail or "AG2" in detail or "AG3" in detail or "AG4" in detail:
                    f = open(f"{_c}/init.conf", "w+")
                    write(f, ["type", "status", "name", "host", "port", "detail", "tk"], detail) # after new version might cert in value.
                    f.close()
                    return os.listdir(_c)
                else:
                    print("[Errno] Create init.conf not success, please check token incorrect.")
            elif is_base64(token) == False and token:
                print("Token incorrect.\nPlease enter again or exit process.")
            else:
                print("Bye.")
                break
        except KeyboardInterrupt:
            print("\nBye.")
            break
        except Exception as e:
            print(str(e))
    else:
        return os.listdir(_c)
#======================================================
# Setup config file
#======================================================
def setup(_c):
    c = []
    try:
        f=open(os.path.join(_c, "init.conf"), "r").readlines()
        for i in f:
            if i.find("#") == -1:
                x=i.split(":=")
                c.append(x[1].strip("\n"))
            else:
                pass
    except Exception as e:
        print(str(e))
        sys.exit(1)
    finally:
        if len(c) < 7:
            print("[Errno] Please check init file.")
            sys.exit(1)
        else:
            return c, c.pop(-1)
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
    c_list = check(os.listdir(__config__), __config__)
    if c_list:
        __init__, __token__ = setup(__config__)
        db = None
        if __init__[0] == "AG3":
            detail = __init__[-1].split("&")
            if detail[0] == 1: # MySQL.
                db = mysql.connector.connect( host=__init__[1], user=__init__[2], password=__init__[3], auth_plugin="mysql_native_password", database=__init__[4] )
            elif detail[0] == 0: # Hold for oracle database.
                pass
            else: # Hold for another service database.
                pass
        else:
            db = None
        while True:
            if __init__[0] != "AG4":
                process = mode.startTask(__config__, __init__, __token__, __ssl__, conn, db)._run()
            else:
                sniffer.taskSnif(__config__, __init__, __token__).run()
            # Delay time.
            time.sleep(5)
    else:
        pass
