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
                    snellen_equivalent=(20.0, 30.0),
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
                    snellen_equivalent=(20.0, 20.0),
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
                        snellen_equivalent=(20.0, 20.0),
                        log_mar_base=0.0,
                        log_mar_base_plus_letters=0.016151668834676065,
                    ),
                    "Both Eyes Near CC": VisitNote(
                        laterality=OU,
                        distance_of_measurement=NEAR,
                        correction=CC,
                        method=JAEGER,
                        extracted_value="J2",
                        snellen_equivalent=(20.0, 25.0),
                        log_mar_base=0.09691001300805639,
                        log_mar_base_plus_letters=0.09691001300805639,
                    )
                }
            ),
            ({}, {}),
        ]
        skip_fields = {"text", "text_plus"}

        for input, expected in test_cases:
            actual = parse_visit(input)
            expected = {
                # Assume text is fine
                key: dataclasses.replace(expected[key], text=val.text, text_plus=val.text_plus)
                for key, val in actual.items()
            }
            for visit_key, actual_note in actual.items():
                expected_note = expected[visit_key]
                for field in dataclasses.fields(actual_note):
                    if field.name in skip_fields:
                        continue
                    with self.subTest((input, f"visit[\"{visit_key}\"].{field.name}")):

                        expected_value = getattr(expected_note, field.name)
                        actual_value = getattr(actual_note, field.name)

                        if field.type == float:
                            expected_value = f"{expected_value:.2f}"
                            actual_value = f"{actual_value:.2f}"
                        elif field.name == "snellen_equivalent":
                            expected_value = "{:.2f}/{:.2f}".format(*expected_value)
                            actual_value = "{:.2f}/{:.2f}".format(*actual_value)

                        self.assertEqual(expected_value, actual_value)

