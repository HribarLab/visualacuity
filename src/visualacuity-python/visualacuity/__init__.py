from visualacuity import _lib
from visualacuity._types import (
    Visit,
    VisitNote,
    Laterality,
    DistanceOfMeasurement,
    Correction,
    Method,
    PinHole,
)
from visualacuity._parse import (
    parse_visit,
)

# Alias some enums for convenience:

OS = Laterality.OS
OD = Laterality.OD
OU = Laterality.OU

NEAR = DistanceOfMeasurement.NEAR
DISTANCE = DistanceOfMeasurement.DISTANCE

CC = Correction.CC
SC = Correction.SC

SNELLEN = Method.SNELLEN
JAEGER = Method.JAEGER
ETDRS = Method.ETDRS
TELLER = Method.TELLER
LOW_VISION = Method.LOW_VISION