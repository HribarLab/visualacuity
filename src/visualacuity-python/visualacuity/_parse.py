from functools import lru_cache
from typing import Dict

from visualacuity import Laterality, DistanceOfMeasurement, Correction, Method, PinHole
from . import _lib, Visit, VisitNote

PARSER = _lib.Parser()


def parse_visit(notes: Dict[str, str]) -> Visit:
    try:
        return Visit({
            key: VisitNote(
                text=val.text,
                text_plus=val.text_plus,
                laterality=_convert_enum(val.laterality),
                distance_of_measurement=_convert_enum(val.distance_of_measurement),
                correction=_convert_enum(val.correction),
                pinhole=_convert_enum(val.pinhole),
                method=_convert_enum(val.method),
                plus_letters=val.plus_letters,
                extracted_value=val.extracted_value,
                snellen_equivalent=_try_get_attr(val, "snellen_equivalent"),
                log_mar_base=_try_get_attr(val, "log_mar_base"),
                log_mar_base_plus_letters=_try_get_attr(val, "log_mar_base_plus_letters"),
            )
            for key, val in PARSER.parse_visit(notes).items()
        })
    except Exception as e:
        raise Exception(f"{e}: `{notes}`") from e


@lru_cache(maxsize=None)
def _convert_enum(enum):
    type_name, value = str(enum).split(".")
    return _CONVERTIBLE_ENUMS[type_name][value]


_CONVERTIBLE_ENUMS = {enum.__name__: enum for enum in (Laterality, DistanceOfMeasurement, Correction, PinHole, Method)}


def _try_get_attr(obj, attr, cast=None):
    try:
        value = getattr(obj, attr)
        return cast(value) if cast else value
    except Exception as e:
        return f"Error"

