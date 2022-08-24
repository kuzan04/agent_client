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
from module import cert, mode, connect, sniffer
#======================================================
# Main
#======================================================
if __name__ == "__main__":
    __location__ = os.path.realpath(os.path.join(os.getcwd(), os.path.dirname(__file__)))
    __config__ = ""
    if "config" in os.listdir(__location__):
        __config__ = os.path.join(__location__, "config")
    else:
        print("[Errno] Please check directory config missing.")
        sys.exit(1)
    DISCONNECT_MESSAGE = "!DISCONNECT"
    while True:
        process = mode.startTask(__location__, __config__)._check()
        if isinstance(process, str):
            config, prepared = eval(process)[0], eval(process)[1]
            if int(prepared[0]) == 0: # FTP
                sniffer.taskSnif(config, prepared[0], 1021, prepared[-1], __config__).run()
            elif int(prepared[0]) == 1: # Syslog
                sniffer.taskSnif(config, prepared[0], 514, prepared[-1], __config__).run()
            else:
                pass
        else:
            print(1)
    '''c = client.SSLClient(
        server_host, server_port, server_sni_hostname, client_cert, client_key
    )
    c.connect()
    c.send("This is a test message!")
    c.close()'''
