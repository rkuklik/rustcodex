#!/usr/bin/env python3
# __SOURCE__
import base64
import gzip
import os
import sys
import tempfile

payload = """__PAYLOAD__"""
binary = gzip.decompress(base64.standard_b64decode(payload))

fd, name = tempfile.mkstemp()
os.write(fd, binary)
os.close(fd)
os.chmod(name, 0o700)

args = list(sys.argv)
args[0] = "binary"
os.execv(name, args)
