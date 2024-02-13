import csv
import os
import re
import sys
import unittest
from typing import Dict, List

from visualacuity import parse_visit


class TestVAInterface(unittest.TestCase):
    maxDiff = None

    def test_cases_conversions_tsv(self):
        filename = "common/test_cases_conversions.tsv"
        test_cases = _load_file(filename)

        for line_number, row in enumerate(test_cases, start=2):
            if row["EHR Entry"].startswith("#"):  # Commented out
                continue

            description = row.pop("Comment")
            expect_snellen_equivalent = row.pop("Snellen Equivalent")
            expect_logmar = row.pop("LogMAR Equivalent")
            expect_logmar_base_plus_letters = row.pop("LogMAR Plus Letters")
            input_plain_text = " ".join(row.values())
            msg = f"`{input_plain_text}` ({os.path.basename(filename)}#{line_number})"

            with self.subTest(f"Line {line_number} - Parsing - {input_plain_text}"):
                output = parse_visit(row)
                actual = output["EHR Entry"]

            try:
                print(
                    f"{input_plain_text}\t"
                    f"{actual.snellen_equivalent[0]}/{actual.snellen_equivalent[1]}\t"
                    f"{actual.log_mar_base}\t"
                    f"{actual.log_mar_base_plus_letters}\t",
                    file=sys.stderr
                )
            except:
                pass

            with self.subTest(f"Line {line_number} - Snellen Equivalent - {input_plain_text}"):
                if expect_snellen_equivalent != "Error":
                    expect_snellen_equivalent = tuple(int(n) for n in expect_snellen_equivalent.split("/"))
                self.assertEqual(
                    expect_snellen_equivalent,
                    actual.snellen_equivalent or "Error",
                    msg=msg
                )

            with self.subTest(f"Line {line_number} - LogMAR Equivalent - {input_plain_text}"):
                expect_logmar = expect_logmar if expect_logmar == "Error" else float(expect_logmar)
                self.assertAlmostEqual(
                    expect_logmar,
                    "Error" if actual.log_mar_base is None else actual.log_mar_base,
                    places=2,
                    msg=msg
                )
            with self.subTest(f"Line {line_number} - LogMAR - {input_plain_text}"):
                self.assertAlmostEqual(
                    _try_float(expect_logmar),
                    _try_float(actual.log_mar_base),
                    places=2,
                    msg=msg
                )

            with self.subTest(f"Line {line_number} - LogMAR Plus Letters - {input_plain_text}"):
                self.assertAlmostEqual(
                    _try_float(expect_logmar_base_plus_letters),
                    _try_float(actual.log_mar_base_plus_letters),
                    places=2,
                    msg=msg
                )

    def test_cases_parsing_tsv(self):
        filename = "common/test_cases_parsing.tsv"
        test_cases = _load_file(filename)

        for line_number, row in enumerate(test_cases, start=2):
            if row["EHR Entry"].startswith("#"):  # Commented out
                continue

            description = row.pop("Comment")
            expect_method = row.pop("Method")
            expect_extracted_value = row.pop("Extracted Value")
            expect_plus_letters = row.pop("Plus Letters")
            input_plain_text = " ".join(row.values())
            msg = f"`{input_plain_text}` ({os.path.basename(filename)}#{line_number})"

            with self.subTest(f"Line {line_number} - Parsing - {input_plain_text}"):
                output = parse_visit(row)
                actual = output["EHR Entry"]

            with self.subTest(f"Line {line_number} - Method - {input_plain_text}"):
                self.assertEqual(
                    expect_method,
                    actual.method.value,
                    msg=msg
                )

            with self.subTest(f"Line {line_number} - Extracted Value - {input_plain_text}"):
                self.assertEqual(
                    expect_extracted_value,
                    actual.extracted_value,
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

    def assertAlmostEqual(self, first, second, *args, places=None, delta=None, **kwargs) -> None:
        if type(first) == type(second):
            super().assertAlmostEqual(first, second, *args, places=places, delta=delta, **kwargs)
        else:
            self.assertEqual(first, second, *args, **kwargs)


def _load_file(filename) -> List[Dict[str, str]]:
    path = os.path.join(os.path.dirname(__file__), filename)
    with open(path) as f:
        reader = csv.DictReader(f, dialect=csv.excel_tab)
        return list(reader)


def _try_float(value):
    if isinstance(value, str):
        value = value.lstrip("+")
    try:
        return float(value)
    except:
        return "Error"
