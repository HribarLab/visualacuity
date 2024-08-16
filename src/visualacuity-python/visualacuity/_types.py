import dataclasses
from dataclasses import dataclass, field
from enum import Enum
from functools import lru_cache
from numbers import Number
from typing import List, Optional, Dict, NamedTuple

from visualacuity._enum_helpers import _FancyEnumMixIn, _OrderedEnumMixIn


class DataQuality(_OrderedEnumMixIn, _FancyEnumMixIn, Enum):
    NO_VALUE = "NoValue"
    EXACT = "Exact"
    MULTIPLE = "Multiple"
    CROSS_REFERENCE = "CrossReference"
    CONVERTIBLE_CONFIDENT = "ConvertibleConfident"
    CONVERTIBLE_FUZZY = "ConvertibleFuzzy"


class Laterality(_FancyEnumMixIn, Enum):
    UNKNOWN = "Unknown"
    OS = "OS"
    OD = "OD"
    OU = "OU"


class DistanceOfMeasurement(_FancyEnumMixIn, Enum):
    UNKNOWN = "Unknown"
    NEAR = "Near"
    DISTANCE = "Distance"


class Correction(_FancyEnumMixIn, Enum):
    UNKNOWN = "Unknown"
    CC = "CC"
    SC = "SC"
    MANIFEST = "Manifest"


class VAFormat(_FancyEnumMixIn, Enum):
    UNKNOWN = "Unknown"
    SNELLEN = "Snellen"
    JAEGER = "Jaeger"
    ETDRS = "ETDRS"
    TELLER = "Teller"
    NEAR_TOTAL_LOSS = "NearTotalLoss"
    VISUAL_RESPONSE = "VisualResponse"
    PIN_HOLE = "PinHole"
    BINOCULAR = "Binocular"
    NOT_TAKEN = "NotTaken"
    CROSS_REFERENCE = "CrossReference"


class PinHole(_FancyEnumMixIn, Enum):
    UNKNOWN = "Unknown"
    WITH = "With"
    WITHOUT = "Without"


class SnellenFraction(NamedTuple):
    distance: Number
    row: Number

    def __lt__(self, other):
        return (self.distance / self.row) < (other.distance / other.row)

    def __gt__(self, other):
        return (self.distance / self.row) > (other.distance / other.row)

    def __str__(self):
        return f"{self.distance}/{self.row}"


@dataclass(unsafe_hash=True)
class VisitNote:
    text: str = ""
    text_plus: str = ""
    data_quality: DataQuality = DataQuality.NO_VALUE
    laterality: Laterality = Laterality.UNKNOWN
    distance_of_measurement: DistanceOfMeasurement = DistanceOfMeasurement.UNKNOWN
    correction: Correction = Correction.UNKNOWN
    pinhole: PinHole = PinHole.UNKNOWN
    va_format: VAFormat = VAFormat.UNKNOWN
    plus_letters: List[int] = field(default_factory=list)
    extracted_value: str = ""
    snellen_equivalent: Optional[SnellenFraction] = None
    log_mar_base: Optional[float] = None
    log_mar_base_plus_letters: Optional[float] = None

    @classmethod
    def build(cls, *args, **kwargs):
        if not args or kwargs:
            return cls(*args, **kwargs)  # force an error
        if args:
            kwargs.update(zip(cls.fields(), args))
        casts = {
            "data_quality": lambda value: DataQuality.get(value),
            "laterality": lambda value: Laterality.get(value, Laterality.UNKNOWN),
            "distance_of_measurement": lambda value: DistanceOfMeasurement.get(value, DistanceOfMeasurement.UNKNOWN),
            "correction": lambda value: Correction.get(value, Correction.UNKNOWN),
            "pinhole": lambda value: PinHole.get(value, PinHole.UNKNOWN),
            "va_format": lambda value: VAFormat.get(value, VAFormat.UNKNOWN),
        }
        for field, cast in casts.items():
            if field in kwargs:
                kwargs[field] = cast(kwargs[field])
        return cls(**kwargs)

    def __iter__(self):
        return (getattr(self, field.name) for field in dataclasses.fields(self))

    def raise_errors(self):
        errors = [
            attr for attr in (self.laterality, self.distance_of_measurement, self.correction, self.va_format, self.pinhole)
            if getattr(attr, "name", attr).upper() == "ERROR"
        ]
        if any(errors):
            raise ValueError(f"Notes had errors: {tuple(getattr(e, 'value', e) for e in errors)}")

    @classmethod
    @lru_cache(maxsize=None)
    def fields(cls):
        return [f.name for f in dataclasses.fields(cls)]


class Visit(Dict[str, VisitNote]):
    """
    A dictionary of `{str: VisitNote}` with some added convenience methods.
    """

    def min(self) -> VisitNote:
        """ The `VisitNote` with the minimum visual acuity value. """
        return self[min(self, key=self._compare_key)]

    def max(self) -> VisitNote:
        """ The `VisitNote` with the maximum visual acuity value. """
        return self[max(self, key=self._compare_key)]

    def _compare_key(self, key):
        if self[key] is None:
            return float("nan")
        value = self[key].log_mar_base_plus_letters
        return value if isinstance(value, float) else float("nan")
