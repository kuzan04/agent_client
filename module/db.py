import sys
import mysql.connector
import cx_Oracle


class dbCheck:
    def __init__(self, t, code, name, conn, tables):
        self._type = t
        self.code = code
        self.name = name
        self._connect = conn
        self._table = tables
        self.message = []

    def queryFromSelected(self, table):
        if int(self._type) == 1:
            cursor=self._connect.cursor()
            table=table.split(":")
            cursor.execute(f"SELECT {table[-1]} FROM {table[0]}")
            result=list(cursor.fetchall())
            message = f"{self.code}#{self.name}|||{table[0]}:{table[-1]}|||{str(result)}"
            if len(message.encode('utf-8')) > 16384:
                total = len(result)
                count = 0
                while total > 50:
                    if count == 0:
                        count += 50
                        self.message.append(f"{self.code}#{self.name}|||{table[0]}:{table[-1]}|||{str(result[:count])}")
                        total -= 50
                    else:
                        ocount = count
                        count += 50
                        self.message.append(f"{self.code}#{self.name}|||{table[0]}:{table[-1]}|||{str(result[ocount:count])}")
                        total -= 50
                else:
                    ocount = count
                    count += total
                    self.message.append(f"{self.code}#{self.name}|||{table[0]}:{table[-1]}|||{str(result[ocount:count])}")
                    total = 0
            else:
                self.message.append(message)
        elif int(self._type) == 0: # Hold oracledb
            pass
        else:
            print("[Errno] Type database not support.")
            sys.exit(1)

    def run(self):
        for i in self._table:
            self.queryFromSelected(i)
        return self.message
