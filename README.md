### License

This software is available under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for more info.

### How it works

Much of the behavior for `visualacuity` is documented in the following spreadsheets, (which are also used for testing 
the software):

* [`testing/test_cases_conversions.tsv`](testing/test_cases_conversions.tsv) demonstrates how the various methods of
  visual acuity measurements are converted into Snellen equivalents and LogMAR values. 
* [`testing/test_cases_parsing.tsv`](testing/test_cases_parsing.tsv) expresses the rules and limitations of converting
  plain-text values into structured objects

### Installation

```bash
pip install visualacuity
```

### Example Usage

```python
import visualacuity
from visualacuity import *

#############
# Basic usage
parsed = visualacuity.parse_visit({
    "Left Eye Distance SC": "20/30 -1",
    "Right Eye Near CC": "J5",
})

assert parsed == {
    "Left Eye Distance SC": VisitNote(
        text="20/30 -1",
        extracted_value="20/30",
        plus_letters=[-1],
        laterality=OS,
        distance_of_measurement=DISTANCE,
        correction=SC,
        method=SNELLEN,
        snellen_equivalent=(20, 30),
        log_mar_base=0.17609125905568127,
        log_mar_base_plus_letters=0.20107900637734125
    ),
    "Right Eye Near CC": VisitNote(
        text="J5",
        extracted_value="J5",
        laterality=OD,
        distance_of_measurement=NEAR,
        correction=CC,
        method=JAEGER,
        snellen_equivalent=(20, 40),
        log_mar_base=0.3010299956639812,
        log_mar_base_plus_letters=0.3010299956639812,
    ),
}

###########################
# "Plus" columns are merged

visit_data = {
    "Both Eyes Near CC": "20/20",
    "Both Eyes Near CC Plus": "+2"
}

parsed = visualacuity.parse_visit(visit_data)

assert parsed == {
    "Both Eyes Near CC": VisitNote(
        text="20/20",
        text_plus="+2",
        laterality=OU,
        distance_of_measurement=NEAR,
        correction=CC,
        method=SNELLEN,
        plus_letters=[+2],
        extracted_value="20/20",
        snellen_equivalent=(20, 20),
        log_mar_base=0.0,
        log_mar_base_plus_letters=-0.041646245536099975
    )
}

```