import sys
from argparse import ArgumentParser

from visualacuity import *
from visualacuity.cli import as_main, TabularCounter, MapReduceLoader, make_dirs_for_file

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
    make_dirs_for_file(out_file)
    stats.to_csv(out_file)
    print(f"Wrote file: {out_file}", file=sys.stderr)


def format_stats(counts: TabularCounter):
    counts = counts.to_dataframe().sort_values("Exact", ascending=False)
    stats = counts.copy().astype("str")
    for idx, row in counts.iterrows():
        stats.loc[idx] = [f"{x:,} ({100 * x / row['Total']:.1f}%)" for x in row]
    stats["Total"] = counts["Total"]
    return stats


class VisualAcuityVisitStatsLoader(MapReduceLoader[TabularCounter, TabularCounter]):
    COLUMNS = ["Total", "Exact", "Convertible", "Unusable"]
    USABILITY = {EXACT: "Exact", CONVERTIBLE_CONFIDENT: "Convertible", CONVERTIBLE_FUZZY: "Convertible"}

    def map(self, visit: Visit) -> TabularCounter:
        counts = TabularCounter(rows=("Any", *visit.keys()))
        for key, entry in visit.items():
            if entry is None:
                continue
            usability = self.USABILITY.get(entry.data_quality, "Unusable")
            counts[key, usability] += 1
            counts["Any", usability] += 1
            counts[key, "Total"] += 1
            counts["Any", "Total"] += 1
        return counts

    def reduce(self, accum: TabularCounter, mapped: TabularCounter) -> TabularCounter:
        if accum is None:
            accum = TabularCounter(index_name="Field", columns=self.COLUMNS)
        accum.update(mapped)
        return accum
