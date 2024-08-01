import dataclasses
from dataclasses import dataclass, field
from enum import Enum
from functools import cache
from numbers import Number
from typing import List, Optional, Dict, NamedTuple

from visualacuity._enum_helpers import _ConciseEnumRepr, _OrderedEnumMixIn


class DataQuality(_OrderedEnumMixIn, _ConciseEnumRepr, Enum):
    NO_VALUE = "NoValue"
    EXACT = "Exact"
    MULTIPLE = "Multiple"
    CROSS_REFERENCE = "CrossReference"
    CONVERTIBLE_CONFIDENT = "ConvertibleConfident"
    CONVERTIBLE_FUZZY = "ConvertibleFuzzy"
    UNUSABLE = "Unusable"


class Laterality(_ConciseEnumRepr, Enum):
    UNKNOWN = "Unknown"
    OS = "OS"
    OD = "OD"
    OU = "OU"


class DistanceOfMeasurement(_ConciseEnumRepr, Enum):
    UNKNOWN = "Unknown"
    NEAR = "Near"
    DISTANCE = "Distance"


class Correction(_ConciseEnumRepr, Enum):
    UNKNOWN = "Unknown"
    CC = "CC"
    SC = "SC"
    MANIFEST = "MANIFEST"


class VAFormat(_ConciseEnumRepr, Enum):
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


class PinHole(_ConciseEnumRepr, Enum):
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

    def __iter__(self):
        return (getattr(self, field.name) for field in dataclasses.fields(self))

    def raise_errors(self):
        errors = [
            attr for attr in (self.laterality, self.distance_of_measurement, self.correction, self.va_format, self.pinhole)
            if attr.name == "ERROR"
        ]
        if any(errors):
            raise ValueError(f"Notes had errors: {tuple(e.value for e in errors)}")

    @classmethod
    @cache
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
