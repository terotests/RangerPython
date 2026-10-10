# SPDX-License-Identifier: MIT
# textwrap for the Ranger Python runtime: dedent, indent, wrap, fill, shorten.

def dedent(text):
    lines = text.split("\n")
    margin = None
    for line in lines:
        stripped = line.lstrip(" \t")
        if not stripped:
            continue
        ws = line[:len(line) - len(stripped)]
        if margin is None:
            margin = ws
        else:
            n = 0
            while n < len(margin) and n < len(ws) and margin[n] == ws[n]:
                n += 1
            margin = margin[:n]
    out = []
    for line in lines:
        if not line.strip(" \t"):
            out.append(line.lstrip(" \t") if line.strip() == "" else line)
        elif margin:
            out.append(line[len(margin):])
        else:
            out.append(line)
    return "\n".join("" if l.strip() == "" else l for l in out)


def indent(text, prefix, predicate=None):
    if predicate is None:
        predicate = lambda line: line.strip()
    out = []
    for line in text.splitlines(True):
        out.append(prefix + line if predicate(line) else line)
    return "".join(out)


def wrap(text, width=70, initial_indent="", subsequent_indent="", break_long_words=True, **kwargs):
    words = text.split()
    lines = []
    cur = initial_indent
    empty = True
    for w in words:
        prefix = initial_indent if not lines else subsequent_indent
        if empty:
            cand = prefix + w
        else:
            cand = cur + " " + w
        if len(cand) <= width or empty and not break_long_words:
            cur = cand
            empty = False
        elif empty:
            while len(prefix + w) > width:
                lines.append(prefix + w[:width - len(prefix)])
                w = w[width - len(prefix):]
                prefix = subsequent_indent
            cur = prefix + w
            empty = False
        else:
            lines.append(cur)
            cur = subsequent_indent + w
    if not empty:
        lines.append(cur)
    return lines


def fill(text, width=70, **kwargs):
    return "\n".join(wrap(text, width, **kwargs))


def shorten(text, width, placeholder=" [...]", **kwargs):
    text = " ".join(text.split())
    if len(text) <= width:
        return text
    words = text.split()
    out = ""
    for w in words:
        cand = w if not out else out + " " + w
        if len(cand) + len(placeholder) > width:
            break
        out = cand
    return out + placeholder
