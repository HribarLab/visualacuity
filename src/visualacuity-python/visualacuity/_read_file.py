# import csv
# import os
# from sys import stderr
# from typing import Iterator, TextIO, Dict
#
# from visualacuity import Visit, parse_visit, VisitNote
#
#
# def parse_file(filename: str, progress=True) -> Iterator[Visit]:
#     with open(filename) as f:
#         reader = _get_reader(f)
#         for line in _progress(reader, desc=f"[{filename}]", progress=progress):
#             yield parse_visit(line)
#
#
# def parse_file_as_dataframe(filename: str, progress=True) -> "DataFrame":
#     try:
#         import pandas
#     except ImportError as e:
#         raise ImportError(f"parse_file_as_dataframe() requires `pandas`") from e
#
#     def iterate():
#         for n, visit in enumerate(parse_file(filename, progress=progress)):
#             for key, note in visit.items():
#                 yield n, key, *note
#
#     columns = ["line", "column", *VisitNote.fields()]
#     return pandas.DataFrame(iterate(), columns=columns).set_index(["line", "column"])
#
#
# def _get_reader(f: TextIO) -> Iterator[Dict[str, str]]:
#     try:
#         _, ext = os.path.splitext(f.name)
#         if f.name.endswith(".csv"):
#             return csv.DictReader(f, dialect=csv.excel)
#         elif f.name.endswith(".tsv"):
#             return csv.DictReader(f, dialect=csv.excel_tab)
#         else:
#             raise NotImplemented(f.name)
#     except NotImplemented:
#         raise
#     except Exception as e:
#         raise NotImplemented(f.name) from e
#
#
# def _progress(iterable, desc=None, total=None, leave=True, file=stderr, *args, progress: bool, **kwargs):
#     if not progress:
#         return iterable
#     try:
#         from tqdm import tqdm
#         return tqdm(iterable, desc=desc, total=total, leave=leave, file=file, *args,**kwargs)
#     except ImportError:
#         print(f"{desc} (Processing...)", file=file)
#         return iterable
