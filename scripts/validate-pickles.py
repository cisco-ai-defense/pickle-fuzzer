#!/usr/bin/env python3
# Copyright 2025 Cisco Systems, Inc. and its affiliates
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0

"""Validate pickle files by attempting to disassemble them."""

from __future__ import annotations

import argparse
import io
import sys
from pathlib import Path
from typing import Iterable, Sequence

import pickletools


DEFAULT_EXTENSIONS: tuple[str, ...] = (".pkl", ".pickle")


def is_confined_to_root(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root)
    except (OSError, RuntimeError, ValueError):
        return False
    return True


def iter_pickles(paths: Sequence[str], extensions: Sequence[str]) -> Iterable[Path]:
    exts = tuple(ext.lower() for ext in extensions)
    for raw in paths:
        path = Path(raw)
        if not path.exists():
            raise FileNotFoundError(f"Path not found: {path}")
        if path.is_dir():
            root = path.resolve()
            yield from (
                child
                for child in path.rglob("*")
                if child.is_file()
                and child.suffix.lower() in exts
                and is_confined_to_root(child, root)
            )
        elif path.is_file():
            if path.suffix.lower() in exts:
                yield path
        else:
            raise ValueError(f"Unsupported path type: {path}")


def validate_pickle(data: bytes) -> tuple[bool, str | None]:
    try:
        disassemble(data)
    except Exception as exc:  # pragma: no cover - defensive
        return False, str(exc)
    return True, None


def disassemble(data: bytes) -> str:
    stop_pos = None
    for _opcode, _arg, pos in pickletools.genops(data):
        stop_pos = pos
    if stop_pos is None:
        raise ValueError("pickle exhausted before seeing STOP")
    if stop_pos + 1 != len(data):
        raise ValueError(f"trailing bytes after STOP: {len(data) - (stop_pos + 1)}")

    buffer = io.StringIO()
    pickletools.dis(data, out=buffer)
    return buffer.getvalue()


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate pickle files by parsing them with pickletools",
    )
    parser.add_argument(
        "paths",
        nargs="+",
        help="Files or directories containing pickles to validate",
    )
    parser.add_argument(
        "--ext",
        "--extension",
        dest="extensions",
        action="append",
        default=None,
        help="File extension to include (default: .pkl, .pickle). Specify multiple times.",
    )
    parser.add_argument(
        "--disassemble",
        choices=("never", "on-error", "always"),
        default="on-error",
        help="Control whether disassembly is printed (default: on-error).",
    )
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="Stop at the first invalid pickle.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    ns = parse_args(sys.argv[1:] if argv is None else argv)
    extensions = ns.extensions if ns.extensions is not None else DEFAULT_EXTENSIONS
    paths = sorted(set(iter_pickles(ns.paths, extensions)))

    if not paths:
        print("No pickle files matched the provided paths and extensions.", file=sys.stderr)
        return 1

    total = len(paths)
    failures = 0

    for path in paths:
        data = path.read_bytes()
        ok, error = validate_pickle(data)
        status = "OK" if ok else "ERROR"
        print(f"[{status}] {path}")

        should_disassemble = ns.disassemble == "always" or (
            ns.disassemble == "on-error" and not ok
        )
        if should_disassemble:
            try:
                print(disassemble(data))
            except Exception as exc:  # pragma: no cover - defensive
                print(f"  <failed to disassemble: {exc}>")

        if not ok:
            failures += 1
            if error:
                print(f"  Reason: {error}")
            if ns.fail_fast:
                break

    print(f"Validated {total} pickle file(s); {failures} failure(s).")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
