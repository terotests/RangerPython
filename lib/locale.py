# SPDX-License-Identifier: MIT
# locale for the Ranger Python runtime: the C locale, always
LC_CTYPE = 0
LC_NUMERIC = 1
LC_TIME = 2
LC_COLLATE = 3
LC_MONETARY = 4
LC_MESSAGES = 5
LC_ALL = 6
CHAR_MAX = 127


class Error(Exception):
    pass


def setlocale(category, locale=None):
    if locale not in (None, "", "C", "POSIX"):
        raise Error("unsupported locale setting")
    return "C"


def getlocale(category=LC_CTYPE):
    return (None, None)


def getpreferredencoding(do_setlocale=True):
    return "UTF-8"


def getencoding():
    return "UTF-8"


def localeconv():
    return {"decimal_point": ".", "thousands_sep": "", "grouping": [], "int_curr_symbol": "",
            "currency_symbol": "", "mon_decimal_point": "", "mon_thousands_sep": "",
            "mon_grouping": [], "positive_sign": "", "negative_sign": "", "int_frac_digits": 127,
            "frac_digits": 127, "p_cs_precedes": 127, "p_sep_by_space": 127, "n_cs_precedes": 127,
            "n_sep_by_space": 127, "p_sign_posn": 127, "n_sign_posn": 127}


def strcoll(a, b):
    return (a > b) - (a < b)


def strxfrm(s):
    return s


def format_string(f, val, grouping=False, monetary=False):
    return f % val


def str(val):
    return "%.12g" % val


def atof(s, func=float):
    return func(s)


def atoi(s):
    return int(s)


def normalize(localename):
    return localename
