import functools

import tdrum

_core = tdrum.Core()

class CoreMeta(type):
    def __new__(cls, name, bases, dct):
        x = super().__new__(cls, name, bases, dct)

        prefix = name[4:].lower() + '_'
        attrs = [a for a in dir(_core)
                 if a.startswith(prefix) and a != prefix + "new"]

        for a in attrs:
            function = getattr(_core, a)
            def method(obj, f, *args, **kwargs):
                args = map(lambda a: getattr(a, 'core_ref', a), args)
                return f(obj.core_ref, *args, **kwargs)

            m = functools.partialmethod(method, function)

            method_name = a[len(prefix):]
            setattr(x, method_name, m)
        return x


class CoreInstrument(metaclass=CoreMeta):
    def __init__(self, name):
        self.core_ref = _core.instrument_new(name)

class CoreFader(metaclass=CoreMeta):
    def __init__(self, name, ref=None):
        self.core_ref = ref or _core.fader_new(name)

master_fader = CoreFader("Master", _core.get_master_fader())

def get_master_fader():
    return master_fader
# f0 = CoreFader("f0")
# f1 = CoreFader("f1")
# f0.test_method(f1, 4711)
