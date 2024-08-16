from argparse import ArgumentParser
from numbers import Number

import visualacuity
from visualacuity import Visit, Laterality, VAFormat
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
    n = counts["Any", "N"]
    counts = counts.to_dataframe()
    stats = counts.map(lambda x: f"{x:,} ({100 * x / n:.1f}%)" if x and n else "0")
    stats["N"] = [f"{n:,}" for n in counts["N"]]
    stats["Pin Hole NI"] = [
        f"{ph_ni:,}/{ph:,} ({100 * ph_ni / ph:.1f}%)" if ph else "0"
        for ph, ph_ni in zip(counts["Pin Hole"], counts["Pin Hole NI"])
    ]
    return stats


class VisualAcuityVisitStatsLoader(MapReduceLoader[TabularCounter, TabularCounter]):
    LATERALITIES = [*(l.name for l in Laterality if l != Laterality.UNKNOWN), "Any"]
    FIELDS = ["N", "Has Text", "Recognized Format", "LogMAR Equivalent", "Distance", "Near", "Manifest", "Pin Hole", "Pin Hole NI"]

    def map(self, visit: Visit) -> TabularCounter:
        counts = TabularCounter()
        for key, entry in visit.items():
            if entry is not None:
                lat = entry.laterality.name

                counts[lat, "Has Text"] = 1
                counts["Any", "Has Text"] = 1

                if entry.va_format == VAFormat.UNKNOWN:
                    continue
                else:
                    counts[lat, "Recognized Format"] = 1
                    counts["Any", "Recognized Format"] = 1
                    if isinstance(entry.log_mar_base, Number) and entry.log_mar_base * 0 == 0:
                        counts[lat, "LogMAR Equivalent"] = 1
                        counts["Any", "LogMAR Equivalent"] = 1
                    if entry.distance_of_measurement == visualacuity.DISTANCE:
                        counts[lat, "Distance"] = 1
                        counts["Any", "Distance"] = 1
                    if entry.distance_of_measurement == visualacuity.NEAR:
                        counts[lat, "Near"] = 1
                        counts["Any", "Near"] = 1
                    if entry.correction == visualacuity.MANIFEST:
                        counts[lat, "Manifest"] = 1
                        counts["Any", "Manifest"] = 1
                    if entry.pinhole == visualacuity.PinHole.WITH:
                        counts[lat, "Pin Hole"] = 1
                        counts["Any", "Pin Hole"] = 1
                        if entry.extracted_value == "NI":  # pretty brittle
                            counts[lat, "Pin Hole NI"] = 1
                            counts["Any", "Pin Hole NI"] = 1
        for lat in self.LATERALITIES:
            counts[lat, "N"] = 1
            # if not counts[lat, "Any VA"]:
            #     counts[lat, "No VA"] = 1
        return counts

    def reduce(self, accum: TabularCounter, mapped: TabularCounter) -> TabularCounter:
        if accum is None:
            accum = TabularCounter(
                index_name="Laterality",
                rows=self.LATERALITIES,
                columns=self.FIELDS
            )
        accum.update(mapped)
        return accum
