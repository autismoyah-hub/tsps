"""Pack authored sequences into a COPY of the cache, so the unmodified client code loads them by sequence id.

usage: packcache.py OUT_CACHE_DIR LOOSE_DIR:SEQ_ID [LOOSE_DIR:SEQ_ID ...]
The source cache is never written. OUT_CACHE_DIR is created as a full copy (dat2 and every idx) and then:
  archive 0   one new group per frame group named by the loose files (frame_G_F.dat -> group G, file F)
  archive 2   group 12 is rewritten with one more file per sequence (file id = the new sequence id)
  archive 255 the reference tables of archives 0 and 2 get the new group / file entries, CRCs and versions
Refuses to replace an existing sequence id or frame group.
What the client checks when it reads a group (disk_cache.rs validate_container): the CRC-32 of the container and the
version in the reference table against the stored bytes and their trailer. Both are written here.
"""
import sys, shutil, zlib, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A


def write_group(root, archive, gid, stored):
    """Append `stored` (container + trailer) to dat2 as a sector chain and point the index entry at it."""
    dat = root / 'main_file_cache.dat2'
    idx = root / f'main_file_cache.idx{archive}'
    assert gid <= 65535, 'this writer uses the 8-byte sector header'
    with dat.open('r+b') as d:
        d.seek(0, 2)
        size = d.tell()
        sector = (size + 519) // 520
        if sector == 0: sector = 1
        d.seek(sector * 520)
        first = sector
        pos = 0; chunk = 0
        while pos < len(stored):
            part = stored[pos:pos + 512]; pos += len(part)
            nxt = sector + 1 if pos < len(stored) else 0
            d.write(gid.to_bytes(2, 'big') + chunk.to_bytes(2, 'big') + nxt.to_bytes(3, 'big') + bytes([archive]) + part.ljust(512, b'\0'))
            sector += 1; chunk += 1
    with idx.open('r+b') as f:
        f.seek(0, 2)
        if f.tell() < gid * 6:
            f.write(bytes(gid * 6 - f.tell()))
        f.seek(gid * 6)
        f.write(len(stored).to_bytes(3, 'big') + first.to_bytes(3, 'big'))


def store(root, cache, archive, gid, files, ref, version=None):
    """files: {file id: bytes}. Writes the group and updates the in-memory reference table `ref`."""
    ids = sorted(files)
    data = A.join_files([files[i] for i in ids])
    container = A.pack_container(data)
    old = ref['groups'].get(gid, {})
    version = version if version is not None else old.get('version', 0) + 1
    entry = dict(old, crc=zlib.crc32(container) & 0xffffffff, version=version, files=ids)
    if ref['flags'] & 4: entry['sizes'] = (len(container), len(data))
    if ref['flags'] & 8: entry['ucrc'] = zlib.crc32(data) & 0xffffffff
    if ref['flags'] & 2: entry['whirlpool'] = bytes(64)
    if ref['flags'] & 1:
        entry.setdefault('name', 0)
        names = dict(zip(old.get('files', []), old.get('file_names', [])))
        entry['file_names'] = [names.get(i, 0) for i in ids]
    ref['groups'][gid] = entry
    write_group(root, archive, gid, container + (version & 0xffffffff).to_bytes(4, "big"))


def store_reference(root, archive, ref, original_raw):
    ref['version'] += 1
    container = A.pack_container(A.build_reference(ref))
    trailer = original_raw[A.container_span(original_raw):]
    write_group(root, 255, archive, container + trailer)


def main():
    out = pathlib.Path(sys.argv[1])
    jobs = []
    for a in sys.argv[2:]:
        d, sid = a.rsplit(':', 1)
        jobs.append((pathlib.Path(d), int(sid)))
    src = A.Cache()
    if out.exists(): shutil.rmtree(out)
    out.mkdir(parents=True)
    for p in src.root.iterdir():
        if p.name.startswith('main_file_cache.'):
            shutil.copy2(p, out / p.name)
    import copy
    ref0, ref2 = copy.deepcopy(src.reference(0)), copy.deepcopy(src.reference(2))
    raw0, raw2 = src.raw(255, 0), src.raw(255, 2)
    # self-check of the reference-table writer on the untouched tables
    for arch, ref, raw in ((0, ref0, raw0), (2, ref2, raw2)):
        assert A.build_reference(ref) == A.unpack(raw), f'reference table {arch} does not re-encode exactly'
    print(f'reference tables 0 and 2 re-encode byte-identical (protocol {ref0["protocol"]}/{ref2["protocol"]}, flags {ref0["flags"]}/{ref2["flags"]})')
    seqs = dict(src.files(2, 12))
    for d, sid in jobs:
        if sid in seqs: raise SystemExit(f'sequence {sid} exists in the cache; choose a free id')
        seq = (d / 'seq.dat').read_bytes()
        s = A.decode_sequence(seq)
        groups = {}
        for p in d.glob('frame_*.dat'):
            _, g, f = p.stem.split('_')
            groups.setdefault(int(g), {})[int(f)] = p.read_bytes()
        for fid in s['frames']:
            g, f = fid >> 16, fid & 65535
            if g in groups:
                assert f in groups[g], f'{d}: seq.dat names frame {g}:{f} but there is no file for it'
            else:
                assert g in ref0['groups'] and f in ref0['groups'][g]['files'], f'{d}: frame {g}:{f} is neither loose nor in the cache'
        for g, files in groups.items():
            if g in ref0['groups']: raise SystemExit(f'frame group {g} exists in the cache; choose a free id')
            store(out, src, 0, g, files, ref0, version=1)
            print(f'archive 0 group {g}: {len(files)} frame files')
        seqs[sid] = seq
        print(f'sequence {sid} <- {d}')
    store(out, src, 2, 12, seqs, ref2)
    store_reference(out, 0, ref0, raw0)
    store_reference(out, 2, ref2, raw2)
    # read back through this file's own reader
    chk = A.Cache(out)
    for d, sid in jobs:
        assert chk.file(2, 12, sid) == (d / 'seq.dat').read_bytes()
        for fid in A.decode_sequence(chk.file(2, 12, sid))['frames']:
            p = d / f'frame_{fid >> 16}_{fid & 65535}.dat'
            if p.exists(): assert chk.file(0, fid >> 16, fid & 65535) == p.read_bytes()
    assert all(chk.file(2, 12, k) == v for k, v in list(src.files(2, 12).items())[::97]), 'old sequences changed'
    print(f'packed cache at {out}: {len(seqs)} sequences; read back OK')


if __name__ == '__main__':
    main()
