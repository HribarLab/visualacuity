import csv
import os
import unittest
from typing import Dict, List

from .helpers import close_enough_visit
from visualacuity import parse_visit, VisitNote, Method


class TestVAInterface(unittest.TestCase):
    maxDiff = None

    def test_cases_conversions_tsv(self):
        filename = "common/test_cases_conversions.tsv"

        for line_number, input, expected_visit in _load_file(filename):
            input_plain_text = " ".join(input.values())

            with self.subTest(f"Line {line_number} - {input_plain_text}"):
                expected = close_enough_visit(expected_visit)
                actual = close_enough_visit(parse_visit(input))

                expected, actual = (
                    {
                        # relevant values to compare
                        "Snellen Equivalent": v.snellen_equivalent,
                        "LogMAR Equivalent": v.log_mar_base,
                        "LogMAR Plus Letters": v.log_mar_base_plus_letters,
                    }
                    for v in (expected["EHR Entry"], actual["EHR Entry"])
                )

                self.assertEqual(actual, expected)

    def test_cases_parsing_tsv(self):
        filename = "common/test_cases_parsing.tsv"

        for line_number, input, expected_visit in _load_file(filename):
            input_plain_text = " ".join(input.values())

            with self.subTest(f"Line {line_number} - Parsing - {input_plain_text}"):
                expected = close_enough_visit(expected_visit)
                actual = close_enough_visit(parse_visit(input))
                expected, actual = (
                    {
                        # relevant values to compare
                        "Method": v.method,
                        "Extracted Value": v.extracted_value,
                        "Plus Letters": v.plus_letters,
                    }
                    for v in (expected["EHR Entry"], actual["EHR Entry"])
                )

                self.assertEqual(actual, expected)


def _load_file(filename) -> List[Dict[str, str]]:
    path = os.path.join(os.path.dirname(__file__), filename)
    with open(path) as f:
        reader = csv.DictReader(f, dialect=csv.excel_tab)
        for line_number, row in enumerate(reader, start=2):
            if row["EHR Entry"].startswith("#"):  # Commented out
                continue

            input = {
                "EHR Entry": row["EHR Entry"],
                "EHR Entry Plus": row["EHR Entry Plus"]
            }

            expected_visit = {
                "EHR Entry": VisitNote(
                    text=row["EHR Entry"],
                    text_plus=row["EHR Entry Plus"],
                    plus_letters=row.get("Plus Letters", []),
                    extracted_value=row.get("Extracted Value", ""),
                    method=Method(row.get("Method", "Unknown")),
                    snellen_equivalent=row.get("Snellen Equivalent", ""),
                    log_mar_base=row.get("LogMAR Equivalent", ""),
                    log_mar_base_plus_letters=row.get("LogMAR Plus Letters", ""),
                )
            }

            yield line_number, input, expected_visit
