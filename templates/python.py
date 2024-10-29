#!/usr/bin/env python3
{}
import base64
import os
import sys
import tempfile
import zlib

payload = """{}"""
binary = zlib.decompress(base64.standard_b64decode(payload))

fd, name = tempfile.mkstemp()
os.write(fd, binary)
os.close(fd)

os.chmod(name, 0o700)

args = list(sys.argv)
args[0] = name
os.execv(name, args)
