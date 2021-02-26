#!/usr/bin/env python3
import uuid
import os

mypath = os.path.dirname(os.path.abspath(__file__))
out_makefile = mypath + "/include/uuid.mk"
out_header = mypath + "/include/uuid.h"

v = uuid.uuid4()
x = list(map(''.join, zip(*[iter(hex(v.fields[5]))] * 2)))[1:]
s = ', '.join([F'0x{item}' for item in x])

with open(out_makefile, "w") as f:
    f.write(F"TA_UUID={v}\n")

with open(out_header, "w") as f:
    f.write(
        F"#pragma once \n\n"
        F"#define TA_UUID {{ \\\n"
        F"  {hex(v.fields[0])} , {hex(v.fields[1])}, {hex(v.fields[2])}, \\\n"
        F"  {{ {hex(v.fields[3])}, {hex(v.fields[4])}, {s} }} \\\n"
        F"}}\n"
    )
