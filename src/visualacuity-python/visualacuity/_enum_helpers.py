from functools import lru_cache


class _OrderedEnumMixIn:
    def __lt__(self, other):
        return self._enum_ordinal(self) < self._enum_ordinal(other)

    def __le__(self, other):
        return self._enum_ordinal(self) <= self._enum_ordinal(other)

    def __gt__(self, other):
        return self._enum_ordinal(self) > self._enum_ordinal(other)

    def __ge__(self, other):
        return self._enum_ordinal(self) >= self._enum_ordinal(other)


class _FancyEnumMixIn:
    def __repr__(self):
        # Concise representation
        return self.name

    @classmethod
    @lru_cache(maxsize=None)
    def _enum_ordinal(cls, obj):
        if not isinstance(obj, cls):
            raise TypeError(f"Expected `{cls.__name__}` for comparison, found `{type(obj).__name__}`")
        return list(cls).index(obj)

    @classmethod
    @lru_cache(maxsize=None)
    def get(cls, value, default=None):
        '''
        Try to get an enum instance from a value
        :param value: input value
        :param default: value to return if cast is not possible
        :return:
        '''
        if isinstance(value, cls):
            return value
        if isinstance(value, str):
            *_, s = value.split(".", maxsplit=1)
            try:
                return cls(s)
            except:
                pass
            try:
                return getattr(cls, s, default)
            except:
                pass
        return default