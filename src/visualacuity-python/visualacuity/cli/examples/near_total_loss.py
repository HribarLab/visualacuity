import os
from argparse import ArgumentParser

from visualacuity import Visit, DISTANCE, CC, NEAR_TOTAL_LOSS
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


def format_stats(counts: TabularCounter):
    counts = counts.to_dataframe().sort_values("N", ascending=False)
    stats = counts.copy()
    columns = [c for c in stats.columns if c != "N"]
    stats["N"] = [f"{n:,}" for n in stats["N"]]
    for column in columns:
        stats[column] = counts[column] / counts["N"]
        stats[column] = [
            f"{num:,} ({100 * num / den:.1f}%)" if den else "0"
            for num, den in zip(counts[column], counts["N"])
        ]
    return stats.rename(columns={"N": "Distance + Corrected"})


class VisualAcuityVisitStatsLoader(MapReduceLoader[TabularCounter, TabularCounter]):
    COLUMNS = ["N", "Near Total Loss", "CF", "HM", "LP", "NLP"]

    def map(self, visit: Visit) -> TabularCounter:
        counts = TabularCounter(rows=("Overall", *visit.keys()))
        for key, entry in visit.items():
            if entry is None:
                continue

            if entry.correction == CC and entry.distance_of_measurement == DISTANCE:
                counts[key, "N"] += 1
                counts["Overall", "N"] += 1
                if entry.va_format == NEAR_TOTAL_LOSS:
                    counts[key, "Near Total Loss"] += 1
                    counts["Overall", "Near Total Loss"] += 1
                for ntl_type in ("CF", "HM", "LP", "NLP"):
                    if entry.extracted_value.startswith(ntl_type):
                        counts[key, ntl_type] += 1
                        counts["Overall", ntl_type] += 1
        return counts

    def reduce(self, accum: TabularCounter, mapped: TabularCounter) -> TabularCounter:
        if accum is None:
            accum = TabularCounter(index_name="Field", columns=self.COLUMNS)
        accum.update(mapped)
        return accum
