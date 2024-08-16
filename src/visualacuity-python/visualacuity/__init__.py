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
    SnellenFraction,
)
from visualacuity._parse import (
    parse_visit,
)

# Alias some enums for convenience:

NO_VALUE = DataQuality.NO_VALUE
EXACT = DataQuality.EXACT
MULTIPLE = DataQuality.MULTIPLE
CROSS_REFERENCE = DataQuality.CROSS_REFERENCE
CONVERTIBLE_CONFIDENT = DataQuality.CONVERTIBLE_CONFIDENT
CONVERTIBLE_FUZZY = DataQuality.CONVERTIBLE_FUZZY

OS = Laterality.OS
OD = Laterality.OD
OU = Laterality.OU

NEAR = DistanceOfMeasurement.NEAR
DISTANCE = DistanceOfMeasurement.DISTANCE

CC = Correction.CC
SC = Correction.SC
MANIFEST = Correction.MANIFEST

SNELLEN = VAFormat.SNELLEN
JAEGER = VAFormat.JAEGER
ETDRS = VAFormat.ETDRS
TELLER = VAFormat.TELLER
NEAR_TOTAL_LOSS = VAFormat.NEAR_TOTAL_LOSS
VISUAL_RESPONSE = VAFormat.VISUAL_RESPONSE
PIN_HOLE = VAFormat.PIN_HOLE
BINOCULAR = VAFormat.BINOCULAR
NOT_TAKEN = VAFormat.NOT_TAKEN

