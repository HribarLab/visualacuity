from dataclasses import dataclass, field
from enum import Enum
from typing import List, Tuple, Optional


class _ConciseEnumRepr:
    def __repr__(self):
        return self.name

class Laterality(_ConciseEnumRepr, Enum):
    ERROR = "Error"
    UNKNOWN = "Unknown"
    OS = "OS"
    OD = "OD"
    OU = "OU"


class DistanceOfMeasurement(_ConciseEnumRepr, Enum):
    ERROR = "Error"
    UNKNOWN = "Unknown"
    NEAR = "Near"
    DISTANCE = "Distance"


class Correction(_ConciseEnumRepr, Enum):
    ERROR = "Error"
    UNKNOWN = "Unknown"
    CC = "CC"
    SC = "SC"


class Method(_ConciseEnumRepr, Enum):
    ERROR = "Error"
    UNKNOWN = "Unknown"
    SNELLEN = "Snellen"
    JAEGER = "Jaeger"
    ETDRS = "ETDRS"
    TELLER = "Teller"
    LOW_VISION = "LowVision"
    PIN_HOLE = "PinHole"
    BINOCULAR = "Binocular"
    NOT_TAKEN = "NotTaken"


class PinHole(_ConciseEnumRepr, Enum):
    ERROR = "Error"
    UNKNOWN = "Unknown"
    WITH = "With"
    WITHOUT = "Without"


@dataclass(unsafe_hash=True)
class VisitNote:
    text: str = ""
    text_plus: str = ""
    laterality: Laterality = Laterality.UNKNOWN
    distance_of_measurement: DistanceOfMeasurement = DistanceOfMeasurement.UNKNOWN
    correction: Correction = Correction.UNKNOWN
    pinhole: PinHole = PinHole.UNKNOWN
    method: Method = Method.UNKNOWN
    plus_letters: List[int] = field(default_factory=list)
    extracted_value: str = ""
    snellen_equivalent: Optional[Tuple[int, int]] = None
    log_mar_base: Optional[float] = None
    log_mar_base_plus_letters: Optional[float] = None

    def raise_errors(self):
        errors = [
            attr for attr in (self.laterality, self.distance_of_measurement, self.correction, self.method, self.pinhole)
            if attr.name == "ERROR"
        ]
        if any(errors):
            raise ValueError(f"Notes had errors: {tuple(e.value for e in errors)}")


ERRORS = {Laterality.ERROR, DistanceOfMeasurement.ERROR, Correction.ERROR, Method.ERROR, PinHole.ERROR}
