import logging
from argparse import ArgumentParser
from enum import Enum
from functools import cache
from numbers import Number

import pandas
import sys

import visualacuity
from examples.map_reduce import MapReduceLoader
from visualacuity import Visit, VisitNote, DataQuality
from visualacuity._enum_helpers import _OrderedEnumMixIn
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
    draw_histogram(distribution, out_file + ".pdf")


class VisualAcuityVisitStatsLoader(MapReduceLoader[pandas.DataFrame, pandas.DataFrame]):
    def map(self, visit: Visit) -> pandas.DataFrame:
        data = [
            [VisualAcuityBin.get(entry), entry.data_quality.value, 1]
            for key, entry in visit.items()
            if entry is not None
        ]
        return pandas.DataFrame(data, columns=["Index", "Data Quality", "Count"])

    def reduce(self, accum: pandas.DataFrame, mapped: pandas.DataFrame) -> pandas.DataFrame:
        if accum is None:
            accum = pandas.DataFrame(index=self.index(), columns=self.columns()).fillna(0).astype("int")
        pivoted = mapped.pivot_table(index="Index", columns="Data Quality", aggfunc="count").droplevel(level=0, axis=1)
        return pandas.concat([accum, pivoted]).groupby(level=0).sum().astype("int")

    def columns(self):
        columns = [
            DataQuality.EXACT.value,
            DataQuality.CONVERTIBLE_CONFIDENT.value,
            DataQuality.CONVERTIBLE_FUZZY.value,
        ]
        return columns + [c.value for c in DataQuality if c.value not in columns]

    def index(self):
        return list(VisualAcuityBin)

    def callback(self, line_num: int, total_lines: int, mapped: pandas.DataFrame, accum: pandas.DataFrame):
        super().callback(line_num, total_lines, mapped, accum)
        if line_num % 100000 == 0:
            accum.to_csv("examples/distribution_temp.csv")
            draw_histogram(accum, "examples/distribution_temp.pdf")


def draw_histogram(stacked: pandas.DataFrame, out_file):
    import matplotlib.pyplot as plt

    n = len(stacked.index)
    figsize = (12, 0.30 * n)
    ax = stacked.plot.barh(
        stacked=True,
        width=0.8,
        figsize=figsize
    )
    ax.invert_yaxis()
    plt.tight_layout()
    plt.savefig(out_file, format="pdf", bbox_inches="tight")
    plt.close()
    # logging.warning(f"Wrote file: {out_file}")


class VisualAcuityBin(_OrderedEnumMixIn, Enum):
    EMPTY = (-1, "")
    S20_10 = (-0.30, "20/10 (–0.30)")
    S20_12 = (-0.20, "20/12.5 (–0.20)")
    S20_16 = (-0.10, "20/16 (–0.10)")
    S20_20 = (0.00, "20/20 (+0.00)")
    S20_25 = (0.10, "20/25 (+0.10)")
    S20_32 = (0.20, "20/32 (+0.20)")
    S20_40 = (0.30, "20/40 (+0.30)")
    S20_50 = (0.40, "20/50 (+0.40)")
    S20_63 = (0.50, "20/63 (+0.50)")
    S20_80 = (0.60, "20/80 (+0.60)")
    S20_100 = (0.70, "20/100 (+0.70)")
    S20_125 = (0.80, "20/125 (+0.80)")
    S20_160 = (0.90, "20/160 (+0.90)")
    S20_200 = (1.00, "20/200 (+1.00)")
    S20_250 = (1.10, "20/250 (+1.10)")
    S20_320 = (1.20, "20/320 (+1.20)")
    S20_400 = (1.30, "20/400 (+1.30)")
    S20_500 = (1.40, "20/500 (+1.40)")
    S20_630 = (1.50, "20/630 (+1.50)")
    S20_800 = (1.60, "20/800 (+1.60)")
    S20_1000 = (1.70, "20/1000 (+1.70)")
    S20_1250 = (1.80, "20/1250 (+1.80)")
    S20_1600 = (1.90, "20/1600 (+1.90)")
    S20_2000 = (2.00, "20/2000 (+2.00)")
    MAX = (2.10, ">20/2000 (>+2.00)")
    CF = (3.00, "CF")
    HM = (4.00, "HM")
    LP = (5.00, "LP")
    NLP = (6.00, "NLP")
    NEAR = (sys.maxsize, "Near VA")
    NI = (sys.maxsize, "NI")
    VISUAL_RESPONSE = (sys.maxsize, "Visual Response")
    UNUSABLE = (sys.maxsize, "Unusable")

    @classmethod
    def get(cls, entry: VisitNote):
        if entry is None:
            return cls.EMPTY
        if not entry.extracted_value:
            return cls.EMPTY
        if entry.distance_of_measurement == visualacuity.NEAR:
            return cls.NEAR
        if entry.va_format == visualacuity.VAFormat.VISUAL_RESPONSE:
            return cls.VISUAL_RESPONSE
        if (
                entry.data_quality == DataQuality.CROSS_REFERENCE
                or entry.va_format == visualacuity.VAFormat.NEAR_TOTAL_LOSS
        ):
            try:
                bin_key, *_ = entry.extracted_value.split()
                return cls[bin_key]
            except:
                text = f"{entry.text} {entry.text_plus}".strip()
                logging.warning(f"Unhandled value: \"{text.strip()}\"")
                return cls.UNUSABLE

        bin_key = entry.log_mar_base
        if isinstance(bin_key, Number):
            bin_key = min(round(bin_key or 0.0, 1), cls.MAX.value[0])
            if entry.data_quality == DataQuality.CONVERTIBLE_FUZZY:
                # print(entry)
                pass

            return cls._by_logmar()[bin_key]

        if entry.data_quality in (DataQuality.NO_VALUE, DataQuality.MULTIPLE,):
            return cls.UNUSABLE
        text = f"{entry.text} {entry.text_plus}".strip()
        logging.warning(f"Unhandled value: \"{text.strip()}\"")
        logging.warning(f"{entry}")
        return cls.UNUSABLE

    @classmethod
    @cache
    def _by_logmar(cls):
        return {x.value[0]: x for x in cls}

    def __str__(self):
        return self.value[1]

