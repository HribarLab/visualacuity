from argparse import ArgumentParser

from visualacuity import Visit, VAFormat
from visualacuity.cli import as_main, TabularCounter, MapReduceLoader

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
    n = counts["Overall", "Snellen"]
    counts = counts.to_dataframe().sort_values("Snellen", ascending=False)
    stats = counts.map(lambda x: f"{x:,} ({100 * x / n:.1f}%)" if x and n else "0")
    stats["Snellen"] = [f"{n:,}" for n in counts["Snellen"]]
    stats["Snellen with +/-"] = [
        f"{num:,}/{den:,} ({100 * num / den:.1f}%)" if den else "0"
        for num, den in zip(counts["With +/-"], counts["Snellen"])
    ]
    return stats[["Snellen with +/-"]]


class VisualAcuityVisitStatsLoader(MapReduceLoader[TabularCounter, TabularCounter]):
    def map(self, visit: Visit) -> TabularCounter:
        counts = TabularCounter(rows=("Overall", *visit.keys()))
        for key, entry in visit.items():
            if entry is None:
                continue

            if entry.va_format == VAFormat.SNELLEN:
                counts[key, "Snellen"] += 1
                counts["Overall", "Snellen"] += 1
                if len(entry.plus_letters) >= 1:
                    counts[key, "With +/-"] += 1
                    counts["Overall", "With +/-"] += 1
        return counts

    def reduce(self, accum: TabularCounter, mapped: TabularCounter) -> TabularCounter:
        if accum is None:
            return mapped
        accum.update(mapped)
        return accum

    def callback(self, line_num: int, total_lines: int, mapped: TabularCounter, accum: TabularCounter):
        super().callback(line_num, total_lines, mapped, accum)
        if line_num % 100000 == 0:
            stats = format_stats(accum)
            stats.to_csv("examples/plus_minus_temp.csv")
