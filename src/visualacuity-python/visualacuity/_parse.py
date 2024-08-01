from functools import cache
from typing import Dict

from visualacuity import Laterality, DistanceOfMeasurement, Correction, VAFormat, PinHole, DataQuality
from . import _lib, Visit, VisitNote

PARSER = _lib.Parser()


def parse_visit(notes: Dict[str, str]) -> Visit:
    def convert_obj(lib_obj):
        if lib_obj is None:
            return None
        else:
            return VisitNote(
                text=lib_obj.text,
                text_plus=lib_obj.text_plus,
                data_quality=_convert_enum(lib_obj.data_quality),
                laterality=_convert_enum(lib_obj.laterality),
                distance_of_measurement=_convert_enum(lib_obj.distance_of_measurement),
                correction=_convert_enum(lib_obj.correction),
                pinhole=_convert_enum(lib_obj.pinhole),
                va_format=_try_get_attr(lib_obj, "va_format", _convert_enum),
                plus_letters=lib_obj.plus_letters,
                extracted_value=lib_obj.extracted_value,
                snellen_equivalent=_try_get_attr(lib_obj, "snellen_equivalent"),
                log_mar_base=_try_get_attr(lib_obj, "log_mar_base"),
                log_mar_base_plus_letters=_try_get_attr(lib_obj, "log_mar_base_plus_letters"),
            )

    try:
        return Visit({
            key: convert_obj(val)
            for key, val in PARSER.parse_visit(notes).items()
        })
    except Exception as e:
        raise Exception(f"{e}: `{notes}`") from e


@cache
def _convert_enum(enum):
    type_name, value = str(enum).split(".")
    if type_name not in _CONVERTIBLE_ENUMS:
        raise TypeError(f"Cannot use enum type: {type_name}")
    try:
        return _CONVERTIBLE_ENUMS[type_name][value]
    except KeyError as e:
        breakpoint()
        raise ValueError(f"Cannot find value: {type_name}.{value}") from e


_CONVERTIBLE_ENUMS = {
    enum.__name__: enum
    for enum in (DataQuality, Laterality, DistanceOfMeasurement, Correction, PinHole, VAFormat)
}


def _try_get_attr(obj, attr, cast=None):
    try:
        value = getattr(obj, attr)
        return cast(value) if cast else value
    except Exception as e:
        return f"Error"
