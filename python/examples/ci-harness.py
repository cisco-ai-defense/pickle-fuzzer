"""minimal Atheris harness for CI smoke tests"""

import sys

import atheris
from pickle_fuzzer.fuzzer import PickleMutator

with atheris.instrument_imports():
    import pickletools


def TestOneInput(data: bytes) -> None:
    if not data:
        return
    proto = data[0] % 6
    mutator = PickleMutator(protocol=proto)
    pickle_bytes = mutator.mutate(data[1:], max_size=256)
    try:
        for _ in pickletools.genops(pickle_bytes):
            pass
    except Exception:
        pass


def main() -> None:
    atheris.Setup(sys.argv, TestOneInput)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
