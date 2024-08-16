from typing import Dict

from . import _lib, Visit

PARSER = _lib.Parser()


def parse_visit(notes: Dict[str, str]) -> Visit:
    return PARSER.parse_visit(notes)


def _try_get_attr(obj, attr, cast=None):
    try:
        value = getattr(obj, attr)
        return cast(value) if cast else value
    except Exception as e:
        return f"Error"
