# Run against an installed r2dnsort build (see TESTING.md), e.g.:
#   maturin develop --features python-ext --release
#   python examples/python_smoke_test.py

import r2dnsort as ns

print("natsorted strings:", ns.natsorted(["a2", "a5", "a9", "a1", "a4", "a10", "a6"]))

print("natsorted mixed numeric:", ns.natsorted([3, 1.5, "a10", "a2"], alg=ns.ns.FLOAT))

print("natsorted bytes:", ns.natsorted([b"a2", b"a10", b"a1"]))

print("natsorted nested tuples:", ns.natsorted([("a", 2), ("a", 10), ("a", 1)]))

print("os_sorted:", ns.os_sorted(["file10.txt", "file2.txt", "file1.txt"]))

print("humansorted:", ns.humansorted(["a2", "a10", "a1"]))

print("realsorted:", ns.realsorted(["-3.5", "2.1", "-1.0"]))

print("natsort_keygen as sort key:", sorted(["a2", "a10", "a1"], key=ns.natsort_keygen()))

items = ["a2", "a10", "a1"]
idx = ns.index_natsorted(items)
print("index_natsorted:", idx)
print("order_by_index:", ns.order_by_index(items, idx))

print("natsorted with reverse:", ns.natsorted(["a2", "a10", "a1"], reverse=True))

print("natsorted with key fn:", ns.natsorted([{"n": "a10"}, {"n": "a2"}], key=lambda d: d["n"]))

print("ALL OK")
