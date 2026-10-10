words = ["alpha", "beta", "gamma", "delta", "epsilon"]
out = []
for i in range(200000):
    w = words[i % 5]
    s = f"{w}-{i}".upper().replace("A", "4")
    if s.startswith("4L"):
        out.append(s[::-1])
    else:
        out.append(s.lower().split("-")[0])
text = " ".join(out)
print(len(text), text.count("4"), text[:40])
