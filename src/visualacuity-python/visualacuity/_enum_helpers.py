from enum import EnumMeta
from functools import cache


class _ConciseEnumRepr:
    def __repr__(self):
        return self.name


class _OrderedEnumMixIn(metaclass=EnumMeta):
    def __lt__(self, other):
        return self._enum_ordinal(self) < self._enum_ordinal(other)

    def __le__(self, other):
        return self._enum_ordinal(self) <= self._enum_ordinal(other)

    def __gt__(self, other):
        return self._enum_ordinal(self) > self._enum_ordinal(other)

    def __ge__(self, other):
        return self._enum_ordinal(self) >= self._enum_ordinal(other)

    @classmethod
    @cache
    def _enum_ordinal(cls, obj):
        if not isinstance(obj, cls):
            raise TypeError(f"Expected `{cls.__name__}` for comparison, found `{type(obj).__name__}`")
        return list(cls).index(obj)
