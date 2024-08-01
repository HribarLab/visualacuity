import logging
from argparse import ArgumentParser

import pandas

import visualacuity
from examples.map_reduce import MapReduceLoader
from visualacuity import Visit, Laterality
from visualacuity.cli import as_main

MAX_LOGMAR = 2.0

ARGS = ArgumentParser()
ARGS.add_argument("filenames", nargs="+")
ARGS.add_argument("out_file")


@as_main(ARGS)
def main(filenames, out_file):
    loader = VisualAcuityVisitStatsLoader()
    distribution = loader.read_csv(*filenames)
    distribution.to_csv(out_file)


def n_lines(filename):
    with open(filename, "rbU") as f:
        for total, _ in enumerate(f):
            pass
        return total


class VisualAcuityVisitStatsLoader(MapReduceLoader[pandas.DataFrame, pandas.DataFrame]):
    def map(self, visit: Visit) -> pandas.DataFrame:
        lateralities = [l.name for l in Laterality] + ["OS && OD", "(OS && OD) || OU"]
        result = {
            lat: {"n": 1, "VA": 0, "Distance": 0, "Near": 0, "Manifest": 0, "Pin Hole": 0, "Pin Hole NI": 0}
            for lat in lateralities
        }
        for key, entry in visit.items():
            if entry is not None:
                if entry.laterality == Laterality.ERROR:
                    logging.warning(entry)

                lat_result = result[entry.laterality.name]
                lat_result["VA"] = 1
                if entry.distance_of_measurement == visualacuity.DISTANCE:
                    lat_result["Distance"] = 1
                if entry.distance_of_measurement == visualacuity.NEAR:
                    lat_result["Near"] = 1
                if entry.correction == visualacuity.MANIFEST:
                    lat_result["Manifest"] = 1
                if entry.pinhole == visualacuity.PinHole.WITH:
                    lat_result["Pin Hole"] = 1
                    if entry.extracted_value == "NI":  # pretty brittle
                        lat_result["Pin Hole NI"] = 1

        result = pandas.DataFrame(result, dtype="bool").rename_axis("Laterality", axis=1).transpose()
        result.loc["OS && OD"] = result.loc["OS"] & result.loc["OD"]
        result.loc["(OS && OD) || OU"] = result.loc["OS && OD"] | result.loc["OU"]
        return result

    def reduce(self, accum: pandas.DataFrame, mapped: pandas.DataFrame) -> pandas.DataFrame:
        mapped = mapped.astype("int")
        if accum is None:
            return mapped
        accum += mapped
        return accum

    def callback(self, line_num: int, total_lines: int, mapped: TMap, accum: TReduce):
        super().callback(line_num, total_lines, mapped, accum)
        if line_num % 1000 == 0:
            accum.to_csv("examples/visit_stats_temp.csv")


