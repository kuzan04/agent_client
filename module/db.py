import sys
import time
import mysql.connector
import cx_Oracle
import datetime

class dbCheck:
    def __init__(self, t, code, name, host, user, passwd, db, tables):
        self._type = t
        self.code = code
        self.name = name
        self._host = host
        self._username = user
        self._password = passwd
        self._database = db
        self._table = tables
        self.message = []

    def queryFromSelected(self, table):
        if int(self._type) == 1:
            db=mysql.connector.connect(
                host=self._host,
                user=self._username,
                password=self._password,
                database=self._database,
                auth_plugin="mysql_native_password"
            )
            cursor=db.cursor()
            table=table.split(":")
            columns=table[-1]
            cursor.execute(f"SELECT {columns} FROM {table[0]}")
            result=list(cursor.fetchall())
            self.message = f"{self.code}#{self.name}|||{table[0]}|||{table[1]}|||{str(result)}"
        elif int(self._type) == 0: # Hold oracledb
            pass
        else:
            print("[Errno] Type database not support.")
            sys.exit(1)

    def run(self):
        for i in self._table:
            self.queryFromSelected(i)
        return self.message
