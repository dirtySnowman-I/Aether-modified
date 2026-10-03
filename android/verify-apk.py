"""Fail packaging if either phone CPU lacks an engine or JNI component."""
from pathlib import Path
import struct
import sys
import zipfile

def elf_load_alignment(b):
    is64 = b[4] == 2
    assert b[:4] == b'\x7fELF' and b[5] == 1
    if is64:
        off = struct.unpack_from('<Q', b, 32)[0]
        size, count = struct.unpack_from('<HH', b, 54)
    else:
        off = struct.unpack_from('<I', b, 28)[0]
        size, count = struct.unpack_from('<HH', b, 42)
    for i in range(count):
        p = off + i * size
        if struct.unpack_from('<I', b, p)[0] == 1:
            yield struct.unpack_from('<Q' if is64 else '<I', b, p + (48 if is64 else 28))[0]

apks = list(Path(sys.argv[1]).glob('*.apk'))
assert apks, 'No APKs were built'
for apk in apks:
    with zipfile.ZipFile(apk) as z:
        abis = {p.split('/')[1] for p in z.namelist() if p.startswith('lib/')}
        for abi in abis:
            for name in ['libaether.so', 'libaether_final_proxy.so', 'libaethertun.so', 'libhev-socks5-tunnel.so']:
                data = z.read(f'lib/{abi}/{name}')
                assert data[:4] == b'\x7fELF', f'Malformed {abi}/{name}'
                if abi == 'arm64-v8a':
                    assert all(a >= 16384 for a in elf_load_alignment(data)), f'4KB-only native component: {name}'
        if 'universal' in apk.name:
            assert abis == {'arm64-v8a', 'armeabi-v7a'}, abis
        print(apk.name, sorted(abis), 'native components verified')
