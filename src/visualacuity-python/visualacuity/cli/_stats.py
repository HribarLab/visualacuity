from collections import Counter
from collections.abc import Mapping
from functools import lru_cache


class TabularCounter(Counter):
    """
    Keeps a count indexed by index/column pairs.
    """

    def __init__(self, iterable=None, *, rows=None, columns=None, index_name=None):
        self.rows = _Index(rows, name=index_name)
        self.columns = _Index(columns)
        super().__init__(iterable)

    def __setitem__(self, key, value):
        return super().__setitem__(self._track_key(key), value)

    def update(self, iterable=None, /, **kwds):
        if isinstance(iterable, Mapping):
            for key in iterable:
                self._track_key(key)
        elif iterable is not None:
            iterable = map(self._track_key, iterable)
        return super().update(iterable, **kwds)

    def _track_key(self, key):
        assert len(key) == 2
        self.rows.track(key[0])
        return self.rows.track(key[0]), self.columns.track(key[1])

    def __repr__(self):
        return repr(self.to_dataframe())

    def to_dataframe(self):
        try:
            import pandas
            df = pandas.DataFrame(0, self.rows, self.columns, dtype="int").rename_axis(self.rows.name, axis=0)
            rows = {r: i for i, r in enumerate(df.index)}
            columns = {c: i for i, c in enumerate(df.columns)}
            for (row, column), value in self.items():
                df.iloc[rows[row], columns[column]] = value
            return df
        except ImportError as e:
            raise ImportError(f"Try `pip install pandas`") from e


class _Index(list):
    def __init__(self, init, *, name=None):
        super().__init__()
        self.name = name
        for item in init or []:
            self.track(item)

    @lru_cache(maxsize=None)
    def track(self, item):
        super().append(item)
        return item

    def __hash__(self):
        return id(self)
