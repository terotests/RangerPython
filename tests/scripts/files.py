import os, os.path
os.makedirs("/tmp/pyrt/out", exist_ok=True)
with open("/tmp/pyrt/out/a.txt", "w") as f:
    f.write("line1\n")
    f.write("line2\n")
with open("/tmp/pyrt/out/a.txt", "a") as f:
    print("line3", file=f)
with open("/tmp/pyrt/out/a.txt") as f:
    print(f.read().splitlines())
with open("/tmp/pyrt/out/a.txt") as f:
    for i, line in enumerate(f):
        print(i, line.rstrip())
print(os.listdir("/tmp/pyrt/out"), os.path.exists("/tmp/pyrt/out/a.txt"), os.path.isdir("/tmp/pyrt"), os.path.isfile("/nope"))
print(os.path.join("/data", "out", "b.csv"), os.path.basename("/x/y/z.txt"), os.path.dirname("/x/y/z.txt"), os.path.splitext("arch.tar.gz"))
rows = [["name", "value"], ["a", 1], ["b", 2]]
with open("/tmp/pyrt/out/t.csv", "w") as f:
    for r in rows:
        f.write(",".join(str(c) for c in r) + "\n")
print(open("/tmp/pyrt/out/t.csv").read())
try:
    open("/missing.txt")
except FileNotFoundError as e:
    print(e)
os.remove("/tmp/pyrt/out/a.txt")
print(sorted(os.listdir("/tmp/pyrt/out")))
os.remove("/tmp/pyrt/out/t.csv")
os.rmdir("/tmp/pyrt/out")
os.rmdir("/tmp/pyrt")
print(os.path.exists("/tmp/pyrt"))
