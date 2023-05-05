#!/usr/bin/python3

import os
import re
import sys
import math


def find_the_secret(pid: int, search_string: str):
    map_file = f"/proc/{pid}/maps"
    mem_file = f"/proc/{pid}/mem"

    if not (os.path.exists(map_file) and os.path.exists(mem_file)):
        print("The PID value of {} is incorrect, exiting.".format(pid), file=sys.stderr)
        sys.exit(1)

    mapping = [0] * 256
    entropy = 0.0
    size = 0
    search_bytes = bytes(search_string, "utf-8")
    with open(map_file, 'r') as map_f, open(mem_file, 'rb', 0) as mem_f:
        uuid_regex = re.compile(b'([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})', re.I)
        for line in map_f.readlines():
            m = re.match(r'([0-9A-Fa-f]+)-([0-9A-Fa-f]+) ([-r])', line)
            start = int(m.group(1), 16)
            end = int(m.group(2), 16)
            try:
                mem_f.seek(start)  # seek to region start
                chunk = mem_f.read(end - start)  # read region contents
                found = uuid_regex.findall(chunk)
                if found:
                    print("UUID found at memory range {}:{}:".format(hex(start), hex(end)))
                    for uuid in found:
                        print("\t{}".format(uuid.decode("utf-8")))
                if len(search_string) > 1 and chunk.find(search_bytes):
                    print("Found {} at memory location {}".format(search_string, hex(start)))
                    print("\t{}".format(chunk.decode("utf-8")))
                for c in chunk:
                    mapping[c] += 1
                size += len(chunk)
            except Exception:
                print(hex(start), '-', hex(end), '[error,skipped]', file=sys.stderr)
                continue
        for b in mapping:
            p = b / float(size)
            if p > 0:
                entropy += -p * math.log(p, 2.0)
        print("Entropy: {:.2f}".format(entropy))


if __name__ == '__main__':
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print("Usage: {} PID <search_string>".format(sys.argv[0]))
        sys.exit(1)

    pid = 0

    try:
        pid = int(sys.argv[1])
    except ValueError:
        print("Invalid pid: {}".format(sys.argv[1]))
        sys.exit(1)

    if len(sys.argv) == 2:
        find_the_secret(pid, "")
    else:
        find_the_secret(pid, sys.argv[2])

# vim: tabstop=8 expandtab shiftwidth=4 softtabstop=4
