<span id="README"></div>
# visualacuity

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

### Example Scripts

#### Prerequisites
For the example scripts, we'll need to install a few more Python packages: `pip install pandas tqdm`

#### Data

Prepare a CSV file with your data. Each row is a "visit," and each column is a field from your EHR. Plus columns will 
be automatically merged. Example data: (document - no pinhole)

Correction:
    * Corrected / CC - ...
    * Uncorrected / SC - ...

Example:

| Distance Corrected Left Eye | Distance Corrected Left Eye +/- | Distance Corrected Right Eye | Distance Corrected Right Eye +/- |
|-----------------------------|---------------------------------|------------------------------|----------------------------------|
| 20/20                       | +1                              | 20/40                        | -2                               |
| CF                          |                                 | HM                           |                                  |


#### Data Quality Report [[source: data_quality.py]](src/visualacuity-python/visualacuity/cli/examples/data_quality.py)

```bash
python -m visualacuity.cli.examples.data_quality "mydata.csv" "reports/data_quality.csv"
```

#### Near / Total Loss Report [[source: near_total_loss.py]](src/visualacuity-python/visualacuity/cli/examples/near_total_loss.py)

```bash
python -m visualacuity.cli.examples.near_total_loss "mydata.csv" "reports/near_total_loss.csv"
```

#### Plus / Minus Values Report [[source: plus_minus.py]](src/visualacuity-python/visualacuity/cli/examples/plus_minus.py)

```bash
python -m visualacuity.cli.examples.plus_minus "mydata.csv" "reports/plus_minus.csv"
```

#### Unexpected Patterns Report [[source: unexpected_patterns.py]](src/visualacuity-python/visualacuity/cli/examples/unexpected_patterns.py)

```bash
python -m visualacuity.cli.examples.unexpected_patterns "mydata.csv" "reports/unexpected_patterns.csv"
```

#### Visit Stats Report [[source: visit_stats.py]](src/visualacuity-python/visualacuity/cli/examples/visit_stats.py)

```bash
python -m visualacuity.cli.examples.visit_stats "mydata.csv" "reports/visit_stats.csv"
```

#### VA Distribution Report [[source: va_distribution.py]](src/visualacuity-python/visualacuity/cli/examples/va_distribution.py)

```bash
python -m visualacuity.cli.examples.va_distribution "mydata.csv" "reports/va_distribution.csv" --plot-file "reports/va_distribution.pdf"
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
        va_format=SNELLEN,
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
        va_format=JAEGER,
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
        va_format=SNELLEN,
        plus_letters=[+2],
        extracted_value="20/20",
        snellen_equivalent=(20, 20),
        log_mar_base=0.0,
        log_mar_base_plus_letters=-0.041646245536099975
    )
}

```

## Contributing

### How to publish to PyPi

1. Bump version in [src/visualacuity-python/Cargo.toml](src/visualacuity-python/Cargo.toml)
2. Commit that file and all other changes to `main` or `staging`
3. Push `main` or `staging` branch to github
4. Check status on [GitHub Actions](https://github.com/HribarLab/visualacuity/actions)
5. If the build is successful, tag the repo with the new version, e.g.: `git tag "python-0.1.0a5"`
    * Prepend `test-` to publish to TestPyPi (e.g. `test-python-0.1.0a5`).
6. Push with `--tags`, e.g.: `git push --tags`
7. Check status on [GitHub Actions](https://github.com/HribarLab/visualacuity/actions)
8. Test new install. (Note: sometimes it doesn't work on the first try — caching or something)
    * TestPyPi: `pip install --index-url https://test.pypi.org/legacy/ visualacuity==0.1.0a5`
    * PyPi: `pip install visualacuity==0.1.0a5`
