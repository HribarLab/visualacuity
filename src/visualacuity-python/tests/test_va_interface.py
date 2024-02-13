import dataclasses
import unittest

from visualacuity import *


class TestVAInterface(unittest.TestCase):
    maxDiff = None

    def test_parse_visit(self):
        test_cases = [
            (
                {"Both Eyes Distance CC": "20/30 -1"},
                {"Both Eyes Distance CC": VisitNote(
                    laterality=OU,
                    distance_of_measurement=DISTANCE,
                    correction=CC,
                    method=SNELLEN,
                    extracted_value="20/30",
                    plus_letters=[-1],
                    snellen_equivalent=(20, 30),
                    log_mar_base=0.17609125905568127,
                    log_mar_base_plus_letters=0.20107900637734125,
                )}
            ),
            (
                {"Both Eyes Distance CC": "20/20", "Both Eyes Distance CC Plus": "+2"},
                {"Both Eyes Distance CC": VisitNote(
                    laterality=OU,
                    distance_of_measurement=DISTANCE,
                    correction=CC,
                    method=SNELLEN,
                    plus_letters=[+2],
                    extracted_value="20/20",
                    snellen_equivalent=(20, 20),
                    log_mar_base=0.0,
                    log_mar_base_plus_letters=-0.041646245536099975,
                )}
            ),
            (
                {
                    "Both Eyes Distance CC": "20/20",
                    "Both Eyes Distance CC +/-": "-2",
                    "Both Eyes Distance SC": "20/20",
                    "Both Eyes Distance SC +/-": "-1",
                    "Both Eyes Near CC": "J2",
                    "Both Eyes Near CC Plus": "",
                    # "Comments": "Forgot glasses today"
                },
                {
                    "Both Eyes Distance CC": VisitNote(
                        laterality=OU,
                        distance_of_measurement=DISTANCE,
                        correction=CC,
                        method=SNELLEN,
                        plus_letters=[-2],
                        extracted_value="20/20",
                        snellen_equivalent=(20, 20),
                        log_mar_base=0.0,
                        log_mar_base_plus_letters=0.03230333766935213,
                    ),
                    "Both Eyes Distance SC": VisitNote(
                        laterality=OU,
                        distance_of_measurement=DISTANCE,
                        correction=SC,
                        method=SNELLEN,
                        plus_letters=[-1],
                        extracted_value="20/20",
                        snellen_equivalent=(20, 20),
                        log_mar_base=0.0,
                        log_mar_base_plus_letters=0.016151668834676065,
                    ),
                    "Both Eyes Near CC": VisitNote(
                        laterality=OU,
                        distance_of_measurement=NEAR,
                        correction=CC,
                        method=JAEGER,
                        extracted_value="J2",
                        snellen_equivalent=(20, 25),
                        log_mar_base=0.09691001300805639,
                        log_mar_base_plus_letters=0.09691001300805639,
                    )
                }
            ),
            ({}, {}),
        ]
        for input, expected in test_cases:
            with self.subTest(input):
                actual = parse_visit(input)
                expected = {
                    # Assume text is fine
                    key: dataclasses.replace(expected[key], text=val.text, text_plus=val.text_plus)
                    for key, val in actual.items()
                }
                self.assertEqual(str(actual), str(expected))
