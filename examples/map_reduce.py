import abc
import csv
import os
from typing import TypeVar, Generic

from tqdm import tqdm

import visualacuity
from visualacuity import Visit


def n_lines(filename):
    with open(filename, "rbU") as f:
        for total, _ in enumerate(f):
            pass
        return total


TMap = TypeVar('TMap')
TReduce = TypeVar('TReduce')


class MapReduceLoader(Generic[TMap, TReduce], metaclass=abc.ABCMeta):
    progress = tqdm(unit="lines")

    @abc.abstractmethod
    def map(self, visit: Visit) -> TMap:
        pass

    @abc.abstractmethod
    def reduce(self, accum: TReduce, mapped: TMap) -> TReduce:
        pass

    def callback(self, line_num: int, total_lines: int, mapped: TMap, accum: TReduce):
        self.progress.total = total_lines
        self.progress.update()

    def read_csv(self, *filenames, callback=None):
        accum = None

        try:
            total = sum(n_lines(filename) for filename in filenames)
        except FileNotFoundError as e:
            raise e

        i = 1

        for filename in filenames:
            filename = os.path.abspath(filename)
            with open(filename) as f:
                reader = csv.DictReader(f)
                for i, row in enumerate(reader, i):
                    parsed = visualacuity.parse_visit(row)
                    mapped = self.map(parsed)
                    accum = self.reduce(accum, mapped)
                    self.callback(i, total, mapped, accum)
        return accum
