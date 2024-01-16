import csv
import os
import re
import unittest
from typing import Dict, List

from visualacuity import parse_visit

FIELD_NAME = "EHR Entry"


class TestVAInterface(unittest.TestCase):
    maxDiff = None

    def test_cases_conversions_tsv(self):
        filename = "common/test_cases_conversions.tsv"
        test_cases = self._load_file(filename)

        for line_number, row in enumerate(test_cases, start=2):
            description = row.pop("Comment")
            expect_snellen_equivalent = row.pop("Snellen Equivalent")
            expect_logmar = row.pop("LogMAR Equivalent")
            input_plain_text = " ".join(row.values())
            msg = f"`{input_plain_text}` ({os.path.basename(filename)}#{line_number})"

            with self.subTest(f"Line {line_number} - Parsing - {input_plain_text}"):
                output = parse_visit(row)
                actual = output[FIELD_NAME]

            with self.subTest(f"Line {line_number} - Snellen Equivalent - {input_plain_text}"):
                self.assertEqual(
                    str(actual.snellen_equivalent),
                    expect_snellen_equivalent,
                    msg=msg
                )

            with self.subTest(f"Line {line_number} - LogMAR Equivalent - {input_plain_text}"):
                expect_logmar = expect_logmar if expect_logmar == "Error" else float(expect_logmar)
                self.assertAlmostEqual(
                    "Error" if actual.log_mar_base is None else actual.log_mar_base,
                    expect_logmar,
                    places=10,
                    msg=msg
                )

    def test_cases_parsing_tsv(self):
        filename = "common/test_cases_parsing.tsv"
        test_cases = self._load_file(filename)

        for line_number, row in enumerate(test_cases, start=2):
            description = row.pop("Comment")
            expect_method = row.pop("Method")
            expect_extracted_value = row.pop("Extracted Value")
            expect_plus_letters = row.pop("Plus Letters")
            input_plain_text = " ".join(row.values())
            msg = f"`{input_plain_text}` ({os.path.basename(filename)}#{line_number})"

            with self.subTest(f"Line {line_number} - Parsing - {input_plain_text}"):
                output = parse_visit(row)
                actual = output[FIELD_NAME]

            with self.subTest(f"Line {line_number} - Method - {input_plain_text}"):
                self.assertEqual(
                    actual.method.value,
                    expect_method,
                    msg=msg
                )

            with self.subTest(f"Line {line_number} - Extracted Value - {input_plain_text}"):
                self.assertEqual(
                    actual.extracted_value,
                    expect_extracted_value,
                    msg=msg
                )

            with self.subTest(f"Line {line_number} - Plus Letters - {input_plain_text}"):
                expect_plus_letters = re.split(r"\s*,\s*", expect_plus_letters) if expect_plus_letters else []
                expect_plus_letters = [int(n) for n in expect_plus_letters]
                self.assertEqual(
                    actual.plus_letters,
                    expect_plus_letters,
                    msg=msg
                )

    def _load_file(self, filename) -> List[Dict[str, str]]:
        path = os.path.join(os.path.dirname(__file__), filename)
        with open(path) as f:
            reader = csv.DictReader(f, dialect=csv.excel_tab)
            return list(reader)
