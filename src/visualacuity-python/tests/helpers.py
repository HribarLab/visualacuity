import dataclasses
from typing import Dict

from visualacuity import VisitNote, SnellenFraction


def close_enough_visit(visit: Dict[str, VisitNote]):
    result = {}
    for visit_key, visit_note in visit.items():
        result[visit_key] =  dataclasses.replace(
            visit_note,
            text="",
            text_plus="",
            va_format=getattr(visit_note.va_format, "value", visit_note.va_format),
            plus_letters=try_float_list(visit_note.plus_letters),
            snellen_equivalent=try_snellen(visit_note.snellen_equivalent),
            log_mar_base=try_float(visit_note.log_mar_base),
            log_mar_base_plus_letters=try_float(visit_note.log_mar_base_plus_letters),
        )
    return result


def try_float(value, precision=2):
    try:
        result, = try_float_list(str(value), precision)
        return result
    except Exception as e:
        return f"Error"


def try_float_list(value, precision=2):
    try:
        if isinstance(value, str):
            value = value.strip("[]")
            value = value.split(",") if len(value) else []
        value = [str(v).strip().lstrip("+") for v in value]
        return [round(float(v), precision) for v in value]
    except Exception as e:
        return f"Error"


def try_snellen(value, precision=2):
    try:
        if isinstance(value, str):
            value = value.split("/", maxsplit=1)
        distance, row = (round(float(n), precision) for n in value)
        return SnellenFraction(distance, row)
    except Exception:
        return f"Error"
