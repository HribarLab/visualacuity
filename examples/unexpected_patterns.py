from argparse import ArgumentParser
from collections import Counter

from visualacuity import Visit, DataQuality
from visualacuity.cli import as_main, TabularCounter, MapReduceLoader

MIN_COUNT = 5

ARGS = ArgumentParser()
ARGS.add_argument(
    "filenames", nargs="+", help="Path(s) to the input file(s)"
)
ARGS.add_argument(
    "out_file", help="Path to save the output file."
)
ARGS.add_argument(
    "--processes", type=int, default=None, help="The number of processes to use for parallel execution."
)


@as_main(ARGS)
def main(filenames, out_file, *, processes=None):
    loader = VisualAcuityVisitStatsLoader(processes=processes)
    counts = loader.read_csv(*filenames)
    stats = format_stats(counts)
    stats.to_csv(out_file)


def format_stats(counts: TabularCounter):
    df = counts.to_dataframe().sort_values(by="Any", ascending=False)
    df = df[df["Any"] >= MIN_COUNT]
    text, dqs, errors = zip(*df.index)
    df.insert(0, "Error", errors)
    df.insert(0, "Data Quality", dqs)
    df = df.rename(index=dict(zip(df.index, text)))
    return df


class VisualAcuityVisitStatsLoader(MapReduceLoader[Counter, Counter]):
    DQ = {
        DataQuality.CONVERTIBLE_FUZZY,
        DataQuality.NO_VALUE,
        DataQuality.MULTIPLE,
        DataQuality.UNUSABLE
    }

    def map(self, visit: Visit) -> Counter:
        counts = TabularCounter(index_name="Parse")
        for column, entry in visit.items():
            if entry is None:
                continue

            text = " ".join(s for s in (entry.text, entry.text_plus) if s)
            if not entry.extracted_value or entry.data_quality in self.DQ:
                row = (text, entry.data_quality.value, "")
                counts[row, "Any"] += 1
                counts[row, column] += 1
            else:
                try:
                    entry.raise_errors()
                except Exception as e:
                    row = (text, entry.data_quality.value, str(e))
                    counts[row, "Any"] += 1
                    counts[row, column] += 1
        return counts

    def reduce(self, accum: TabularCounter, mapped: TabularCounter) -> TabularCounter:
        if accum is None:
            accum = TabularCounter(index_name="Text")
        accum.update(mapped)
        return accum

    def callback(self, line_num: int, total_lines: int, mapped: TabularCounter, accum: TabularCounter):
        super().callback(line_num, total_lines, mapped, accum)
        if line_num % 100000 == 0:
            stats = format_stats(accum)
            stats.to_csv("examples/unexpected_patterns_temp.csv")
