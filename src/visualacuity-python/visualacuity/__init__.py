from visualacuity import _lib
from visualacuity._types import (
    Visit,
    VisitNote,
    DataQuality,
    Laterality,
    DistanceOfMeasurement,
    Correction,
    VAFormat,
    PinHole,
)
from visualacuity._parse import (
    parse_visit,
)

# Alias some enums for convenience:

EXACT = DataQuality.EXACT
CONVERTIBLE = DataQuality.CONVERTIBLE
UNRECOGNIZED = DataQuality.UNRECOGNIZED

OS = Laterality.OS
OD = Laterality.OD
OU = Laterality.OU

NEAR = DistanceOfMeasurement.NEAR
DISTANCE = DistanceOfMeasurement.DISTANCE

CC = Correction.CC
SC = Correction.SC

SNELLEN = VAFormat.SNELLEN
JAEGER = VAFormat.JAEGER
ETDRS = VAFormat.ETDRS
TELLER = VAFormat.TELLER
LOW_VISION = VAFormat.NEAR_TOTAL_LOSS