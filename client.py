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
from module import cert, mode, connect, sniffer
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
    # ==================================================
    # Connection Main Database
    # ==================================================
    conn = mysql.connector.connect(
        host="127.0.0.1",
        user="root",
        password="P@ssw0rd",
        auth_plugin="mysql_native_password",
        database="DOL_PDPA_LOCAL"
    )
    while True:
        process = mode.startTask(__location__, __config__, __ssl__, conn)._check()
        if isinstance(process, str):
            config, prepared = eval(process)[0], eval(process)[1]
            if int(prepared[0]) == 0: # FTP
                sniffer.taskSnif(config, prepared[0], 21, prepared[-1], __config__).run()
            elif int(prepared[0]) == 1: # Syslog
                sniffer.taskSnif(config, prepared[0], 514, prepared[-1], __config__).run()
            else:
                pass
