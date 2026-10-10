# generates rust/src/unidata.rs from this Python's unicodedata:
#   python3 tools/unigen.py > src/unidata.rs
import sys, unicodedata
N = 0x110000
def ranges(pred):
    out = []; start = None
    for c in range(N):
        ch = chr(c)
        ok = pred(ch)
        if ok and start is None: start = c
        if not ok and start is not None:
            out.append((start, c - 1)); start = None
    if start is not None: out.append((start, N - 1))
    return out
def emit(name, rs):
    body = ", ".join("(0x%x, 0x%x)" % r for r in rs)
    return "pub const %s: [(u32, u32); %d] = [%s];\n" % (name, len(rs), body)
def esc(s):
    return "".join("\\u{%x}" % ord(c) for c in s)
def emit_map(name, items):
    body = ", ".join('(0x%x, "%s")' % (c, esc(s)) for c, s in items)
    return "pub const %s: [(u32, &str); %d] = [%s];\n" % (name, len(items), body)
def single(m):
    return lambda ch: getattr(ch, m)()
out = ["// SPDX-License-Identifier: MIT\n// Generated from Unicode %s character data by the Python of the same\n// version (unicodedata); character classes as str's is* methods see them.\n\n" % unicodedata.unidata_version]
for name, m in [("ALPHA", "isalpha"), ("DIGIT", "isdigit"), ("NUMERIC", "isnumeric"), ("PRINTABLE", "isprintable"), ("SPACE", "isspace"), ("LOWER", "islower"), ("UPPER", "isupper")]:
    out.append(emit(name, ranges(single(m))))
out.append(emit("TITLE", ranges(lambda ch: unicodedata.category(ch) == "Lt")))
out.append(emit("CASED", ranges(lambda ch: ch.islower() or ch.isupper() or ch.istitle() or unicodedata.category(ch) in ("Lu", "Ll", "Lt"))))
out.append(emit("CASE_IGNORABLE", ranges(lambda ch: unicodedata.category(ch) in ("Mn", "Me", "Cf", "Lm", "Sk") or ch in "'.:^`­··՟״‘’․‧︓﹒﹕＇．：")))
out.append(emit("XID_START", ranges(lambda ch: ch.isidentifier())))
out.append(emit("XID_CONTINUE", ranges(lambda ch: ("a" + ch).isidentifier())))
out.append(emit_map("TITLE_MAP", [(c, chr(c).title()) for c in range(N) if chr(c).title() != chr(c).upper()]))
out.append(emit_map("UPPER_MAP", [(c, chr(c).upper()) for c in range(N) if chr(c).upper() != chr(c)]))
out.append(emit_map("LOWER_MAP", [(c, chr(c).lower()) for c in range(N) if chr(c).lower() != chr(c)]))
out.append(emit_map("FOLD_MAP", [(c, chr(c).casefold()) for c in range(N) if chr(c).casefold() != chr(c).lower()]))
out.append("""
/// c is in one of the sorted ranges
pub fn is_in(t: &[(u32, u32)], c: char) -> bool {
    let u = c as u32;
    let i = t.partition_point(|r| r.1 < u);
    i < t.len() && t[i].0 <= u
}

/// str.upper of one character
pub fn push_upper(out: &mut String, c: char) {
    match mapped(&UPPER_MAP, c) {
        Some(m) => out.push_str(m),
        None => out.push(c),
    }
}

/// str.lower of one character, out of context
pub fn push_lower(out: &mut String, c: char) {
    match mapped(&LOWER_MAP, c) {
        Some(m) => out.push_str(m),
        None => out.push(c),
    }
}

/// the mapping of c, when the table has one
pub fn mapped(t: &[(u32, &'static str)], c: char) -> Option<&'static str> {
    t.binary_search_by_key(&(c as u32), |e| e.0).ok().map(|i| t[i].1)
}
""")
sys.stdout.write("".join(out))
