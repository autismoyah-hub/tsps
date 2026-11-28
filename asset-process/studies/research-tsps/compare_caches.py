"""Read-only comparison of two OSRS dat2 caches (239 project cache vs TSPS 241 cache).
Usage: python -I compare_caches.py CACHE_A CACHE_B
Prints, per cache: reference-table protocol/flags, group counts, max ids, version-trailer size,
config file counts and the set of opcodes used by item/npc/loc/seq/spotanim definitions,
model trailer markers, and the layout of map groups."""
import bz2, gzip, io, os, struct, sys, zlib
from collections import Counter


class Cache:
    def __init__(self, d):
        self.d = d
        self.dat = open(os.path.join(d, "main_file_cache.dat2"), "rb")
        self.idx = {}

    def raw(self, archive, group):
        if archive not in self.idx:
            p = os.path.join(self.d, "main_file_cache.idx%d" % archive)
            self.idx[archive] = open(p, "rb").read() if os.path.exists(p) else b""
        ix = self.idx[archive]
        if group * 6 + 6 > len(ix):
            return None
        e = ix[group * 6:group * 6 + 6]
        length = int.from_bytes(e[:3], "big")
        sector = int.from_bytes(e[3:], "big")
        if length == 0 or sector == 0:
            return None
        out = bytearray()
        big = group > 0xFFFF
        hdr = 10 if big else 8
        while len(out) < length:
            self.dat.seek(sector * 520)
            s = self.dat.read(520)
            sector = int.from_bytes(s[hdr - 4:hdr - 1], "big")
            out += s[hdr:hdr + min(520 - hdr, length - len(out))]
        return bytes(out)

    def unpack(self, archive, group):
        r = self.raw(archive, group)
        if r is None:
            return None, None
        comp = r[0]
        n = int.from_bytes(r[1:5], "big")
        if comp == 0:
            return r[5:5 + n], len(r) - (5 + n)
        body = r[9:9 + n]
        trailer = len(r) - (9 + n)
        if comp == 1:
            return bz2.decompress(b"BZh1" + body), trailer
        return zlib.decompress(body[10:], -15), trailer


def reftable(c, archive):
    data, trailer = c.unpack(255, archive)
    if data is None:
        return None
    b = io.BytesIO(data)
    rd = lambda f: struct.unpack(">" + f, b.read(struct.calcsize(">" + f)))[0]
    protocol = rd("B")
    if protocol >= 6:
        rd("i")
    flags = rd("B")

    def big():
        p = b.read(1)[0]
        if p & 0x80:
            return ((p & 0x7F) << 24) | int.from_bytes(b.read(3), "big")
        return (p << 8) | b.read(1)[0]
    cnt = big() if protocol >= 7 else rd("H")
    ids, last = [], 0
    for _ in range(cnt):
        last += big() if protocol >= 7 else rd("H")
        ids.append(last)
    if flags & 1:
        b.read(4 * cnt)
    b.read(4 * cnt)  # crc
    if flags & 8:
        b.read(4 * cnt)
    if flags & 2:
        b.read(64 * cnt)
    if flags & 4:
        b.read(8 * cnt)
    b.read(4 * cnt)  # versions
    fc = [big() if protocol >= 7 else rd("H") for _ in range(cnt)]
    files = {}
    for g, n in zip(ids, fc):
        last, fl = 0, []
        for _ in range(n):
            last += big() if protocol >= 7 else rd("H")
            fl.append(last)
        files[g] = fl
    return {"protocol": protocol, "flags": flags, "ids": ids, "files": files, "trailer": trailer}


def split(data, n):
    if n == 1:
        return [data]
    chunks = data[-1]
    pos = len(data) - 1 - chunks * n * 4
    sizes = [0] * n
    per = []
    for _ in range(chunks):
        s, row = 0, []
        for i in range(n):
            s += struct.unpack(">i", data[pos:pos + 4])[0]
            pos += 4
            row.append(s)
            sizes[i] += s
        per.append(row)
    out = [bytearray() for _ in range(n)]
    p = 0
    for row in per:
        for i, s in enumerate(row):
            out[i] += data[p:p + s]
            p += s
    return [bytes(x) for x in out]


def report(path):
    c = Cache(path)
    print("==== %s" % path)
    tables = {}
    for a in (0, 1, 2, 5, 7, 8):
        t = reftable(c, a)
        tables[a] = t
        if not t:
            print("archive %d: no table" % a)
            continue
        print("archive %d: protocol %d flags %d groups %d max id %d table-trailer %s" % (
            a, t["protocol"], t["flags"], len(t["ids"]), max(t["ids"]), t["trailer"]))
    tr = Counter()
    for a in (0, 2, 5, 7):
        for g in tables[a]["ids"][:40]:
            _, x = c.unpack(a, g)
            tr[x] += 1
    print("group version-trailer sizes (sample):", dict(tr))
    t2 = tables[2]
    names = {1: "underlay", 4: "overlay", 6: "loc", 9: "npc", 10: "obj", 12: "seq", 13: "spotanim"}
    for g, nm in names.items():
        fl = t2["files"].get(g)
        if not fl:
            print("config %s: missing" % nm)
            continue
        print("config %-8s files %6d max id %6d" % (nm, len(fl), max(fl)))
    # model trailer markers
    mk = Counter()
    t7 = tables[7]
    step = max(1, len(t7["ids"]) // 4000)
    for g in t7["ids"][::step]:
        d, _ = c.unpack(7, g)
        if d and len(d) >= 2:
            mk[(d[-2], d[-1]) if d[-2] == 0xFF and d[-1] >= 0xFC else "old"] += 1
    print("model format markers (last two bytes, sample of %d):" % sum(mk.values()),
          {("%02x%02x" % k if k != "old" else k): v for k, v in mk.items()})
    # newest models only
    mk2 = Counter()
    for g in t7["ids"][-1500:]:
        d, _ = c.unpack(7, g)
        if d and len(d) >= 2:
            mk2[(d[-2], d[-1]) if d[-2] == 0xFF and d[-1] >= 0xFC else "old"] += 1
    print("model format markers, newest 1500:", {("%02x%02x" % k if k != "old" else k): v for k, v in mk2.items()})
    # maps
    t5 = tables[5]
    fcounts = Counter(tuple(v) for v in t5["files"].values())
    print("map groups: %d, flags %d (bit0 = named groups), file-id layouts (top 5): %s" % (
        len(t5["ids"]), t5["flags"], fcounts.most_common(5)))
    # frames: first two bytes = base id; archive 1 size
    print("frame groups %d (max %d), frame bases %d (max %d)" % (
        len(tables[0]["ids"]), max(tables[0]["ids"]), len(tables[1]["ids"]), max(tables[1]["ids"])))
    return c, tables


def first_opcodes(c, tables, group):
    """Set of first-level opcodes cannot be had without a decoder; return raw files instead."""
    fl = tables[2]["files"][group]
    data, _ = c.unpack(2, group)
    return dict(zip(fl, split(data, len(fl))))


if __name__ == "__main__":
    a, ta = report(sys.argv[1])
    b, tb = report(sys.argv[2])
    print("==== definitions byte-identical in both caches (same id present in both)")
    for g, nm in ((1, "underlay"), (6, "loc"), (9, "npc"), (10, "obj"), (12, "seq"), (13, "spotanim")):
        fa, fb = first_opcodes(a, ta, g), first_opcodes(b, tb, g)
        common = set(fa) & set(fb)
        same = sum(1 for i in common if fa[i] == fb[i])
        print("%-8s common ids %6d identical bytes %6d (%.1f%%)" % (nm, len(common), same, 100.0 * same / max(1, len(common))))
    print("==== models / frames / bases byte-identical (sampled ids present in both)")
    for arch, nm in ((7, "model"), (0, "frame group"), (1, "frame base")):
        common = sorted(set(ta[arch]["ids"]) & set(tb[arch]["ids"]))
        step = max(1, len(common) // 1500)
        sample = common[::step]
        same = 0
        for g in sample:
            da, _ = a.unpack(arch, g)
            db, _ = b.unpack(arch, g)
            same += da == db
        print("%-12s common %6d sampled %5d identical %5d" % (nm, len(common), len(sample), same))
