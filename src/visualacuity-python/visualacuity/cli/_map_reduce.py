import abc
import csv
import json
import logging
import multiprocessing
from functools import partial, lru_cache
from typing import TypeVar, Generic, Dict

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

    def __init__(self, preprocessed: bool = False, processes: int = None):
        self.preprocessed = preprocessed
        self.processes = processes

    @property
    @lru_cache(maxsize=None)
    def progress(self):
        try:
            from tqdm import tqdm
            return tqdm(unit="lines")
        except ImportError:
            logging.warning(f"If you'd like a progress bar, do `pip install tqdm`")
            return None

    def parse(self, row: Dict[str, str]) -> Visit:
        if self.preprocessed:
            return Visit({
                key: visualacuity.VisitNote.build(*json.loads(serialized)) if serialized else None
                for key, serialized in row.items()
            })
        else:
            return visualacuity.parse_visit(row)

    @abc.abstractmethod
    def map(self, visit: Visit) -> TMap:
        pass

    @abc.abstractmethod
    def reduce(self, accum: TReduce, mapped: TMap) -> TReduce:
        pass

    def read_csv(self, *filenames):
        accum = None
        reader = MultiCsvReader(filenames, progress=self.progress)
        jobs = self._maybe_parallel(self._parse_map, reader)
        for i, mapped in enumerate(jobs, start=1):
            accum = self.reduce(accum, mapped)
        return accum

    def _parse_map(self, row):
        return self.map(self.parse(row))

    @property
    def _maybe_parallel(self):
        if self.processes:
            return partial(multiprocessing.Pool(self.processes).imap, chunksize=1000)
        else:
            return map


class MultiCsvReader:
    def __init__(self, filenames, progress):
        self.filenames = filenames
        self.progress = progress

    def __iter__(self):
        total = self._get_total_lines()
        self.update_progress(0, 0, 0, total)
        overall_number = 1
        for file_number, filename in enumerate(self.filenames, start=1):
            with open(filename) as f:
                reader = csv.DictReader(f)
                for overall_number, row in enumerate(reader, start=overall_number):
                    self.update_progress(file_number, reader.line_num, overall_number, total)
                    yield row
        if self.progress:
            self.progress.close()

    def update_progress(self, file_number, file_line_number, overall_number, total):
        if self.progress is None:
            return
        self.progress.unit = "lines"
        self.progress.total = total
        self.progress.n = overall_number
        self.progress.set_description(f"File {file_number}/{len(self.filenames)}")

    def _get_total_lines(self):
        try:
            total = 0
            for filename in self.filenames:
                with open(filename, "rbU") as f:
                    for total, _ in enumerate(f, start=total):
                        pass
            return total
        except FileNotFoundError as e:
            return None

