from OpenSSL import crypto, SSL

class createCert:
    def __init__(self, init, key):
        self._format = "utf-8"
        self.emailAddress = init[0]
        self.commonname = init[1]
        self.countryName = init[2]
        self.localityName = init[3]
        self.stateOrProvinceName = init[-3]
        self.organizationName = init[-2]
        self.organizationUnitName = init[-1]
        self.serialNumber = 0
        self.validaityStartInSeconds = 0
        self.validaityEndInseconds = 10*365*24*60*60
        self._key = f"{key}.key"
        self._cert = f"{key}.crt"

    def gen(self):
        k = crypto.PKey()
        k.generate_key(crypto.TYPE_RSA, 2048)
        cert = crypto.X509()
        cert.get_subject().C = self.countryName
        cert.get_subject().ST = self.stateOrProvinceName
        cert.get_subject().L = self.localityName
        cert.get_subject().O = self.organizationName
        cert.get_subject().OU = self.organizationUnitName
        cert.get_subject().CN = self.commonname
        cert.get_subject().emailAddress = self.emailAddress
        cert.set_serial_number(self.serialNumber)
        cert.gmtime_adj_notBefore(self.validaityStartInSeconds)
        cert.gmtime_adj_notAfter(self.validaityEndInseconds)
        cert.set_issuer(cert.get_subject())
        cert.set_pubkey(k)
        cert.sign(k, 'sha256')
        with open(self._cert, "wt") as f:
            f.write(crypto.dump_certificate(crypto.FILETYPE_PEM, cert).decode(self._format))
            f.close()
        with open(self._key, "wt") as f:
            f.write(crypto.dump_privatekey(crypto.FILETYPE_PEM, k).decode(self._format))
            f.close()
