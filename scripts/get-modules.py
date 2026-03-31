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

"""Build stdlib GLOBAL targets for the generator catalog.

The generator emits ``GLOBAL`` and ``INST`` operands as ``module\\nname\\n``.
This script therefore discovers stdlib modules and records only module/member
pairs that this lookup form can resolve. Package discovery uses filesystem
enumeration so the builder does not import package trees just to discover
submodules.

Entries are stored as ``module<TAB>member`` in ``data/stdlib_complete.txt``.
"""

from __future__ import annotations

import importlib
import importlib.util
import os
import pkgutil
import sys
from pathlib import Path

# Configuration
# Modules with side effects or that are slow/problematic to introspect
SKIP_IMPORT = {
    "antigravity",  # Opens web browser
    "this",  # Prints zen of Python
}

# Modules that hang or are very slow during introspection
SKIP_INTROSPECT = {
    "tkinter",  # GUI - can hang
    "turtle",  # GUI - can hang
    "turtledemo",  # GUI demos
    "idlelib",  # IDE library - can be slow
    "test",  # Test suite - very large
    "pydoc",  # Can open help browser
    "webbrowser",  # Can open browser
    "lib2to3",  # Large parsing library - slow
}

ENTRY_SEPARATOR = "\t"
OUTPUT_FILE = Path(__file__).resolve().parent.parent / "data" / "stdlib_complete.txt"


def get_stdlib_base_module_names() -> set[str]:
    """Return the complete stdlib module surface for supported interpreters."""
    stdlib_base = getattr(sys, "stdlib_module_names", None)
    if stdlib_base is None:
        raise RuntimeError(
            "scripts/get-modules.py requires Python with sys.stdlib_module_names "
            "(Python 3.10+; project pin is 3.11.x). Refusing to fall back to "
            "sys.builtin_module_names because it omits most of the standard library."
        )
    return set(stdlib_base)


def should_skip_introspection(module_name: str) -> bool:
    """Return whether a module or package tree should be left uninterpreted."""
    for skip_prefix in SKIP_INTROSPECT:
        if module_name == skip_prefix or module_name.startswith(skip_prefix + "."):
            return True
    return False


def get_package_submodule_names(package_name: str) -> set[str]:
    """Discover submodules without importing the package tree."""
    try:
        spec = importlib.util.find_spec(package_name)
    except (AttributeError, ImportError, ValueError):
        return set()

    if spec is None or spec.submodule_search_locations is None:
        return set()

    all_names = set()
    pending = [(package_name, [os.fspath(path) for path in spec.submodule_search_locations])]
    visited = set()

    while pending:
        current_name, current_paths = pending.pop()
        visit_key = (current_name, tuple(sorted(current_paths)))
        if visit_key in visited:
            continue
        visited.add(visit_key)

        try:
            discovered = list(
                pkgutil.iter_modules(current_paths, prefix=current_name + ".")
            )
        except Exception:
            continue

        for module_info in discovered:
            all_names.add(module_info.name)
            if not module_info.ispkg or should_skip_introspection(module_info.name):
                continue

            child_name = module_info.name.rsplit(".", 1)[-1]
            child_paths = []
            for current_path in current_paths:
                child_path = os.path.join(current_path, child_name)
                if os.path.isdir(child_path):
                    child_paths.append(child_path)
            if child_paths:
                pending.append((module_info.name, child_paths))

    return all_names


def get_all_module_names() -> set[str]:
    """Collect stdlib modules and submodules safe to introspect."""
    all_names = set()

    for mod_name in get_stdlib_base_module_names():
        if mod_name in SKIP_IMPORT or should_skip_introspection(mod_name):
            continue

        all_names.add(mod_name)
        all_names.update(get_package_submodule_names(mod_name))

    return all_names


def format_catalog_entry(module_name: str, attr_name: str) -> str:
    """Encode a module/member pair without ambiguity for nested module paths."""
    return f"{module_name}{ENTRY_SEPARATOR}{attr_name}"


def get_module_members(module_name: str) -> set[str]:
    """Collect public module-level members usable as GLOBAL targets."""
    members = set()

    if module_name in SKIP_IMPORT:
        return members

    # Skip __main__ submodules (they often hang or have side effects)
    if "__main__" in module_name:
        return members

    if should_skip_introspection(module_name):
        return members

    try:
        mod = importlib.import_module(module_name)
    except Exception:
        return members

    try:
        all_attrs = dir(mod)
    except Exception:
        return members

    for attr_name in all_attrs:
        if attr_name.startswith("__") and attr_name.endswith("__"):
            continue

        try:
            getattr(mod, attr_name)
        except Exception:
            continue

        # GLOBAL/INST resolve module-level names only; nested class members do not.
        members.add(format_catalog_entry(module_name, attr_name))

    return members


def main() -> None:
    print("Collecting all stdlib module names...")
    all_modules = get_all_module_names()
    print(f"Found {len(all_modules)} modules")

    print("\nIntrospecting modules for GLOBAL-resolvable members...")
    all_items = set()

    total = len(all_modules)
    for idx, mod_name in enumerate(sorted(all_modules), 1):
        if idx % 50 == 0 or idx > 500:
            print(f"  Processed {idx}/{total} modules... (current: {mod_name})")

        try:
            members = get_module_members(mod_name)
            all_items.update(members)
        except Exception as e:
            print(f"  Warning: Failed to process {mod_name}: {e}")

    print(f"\nCollected {len(all_items)} total items")

    print(f"Writing to {OUTPUT_FILE}...")
    with OUTPUT_FILE.open("w", encoding="utf-8") as f:
        for item in sorted(all_items):
            if "__main__" in item:
                continue
            f.write(f"{item}\n")

    print(f"\nDone! Written {len(all_items)} items to {OUTPUT_FILE}")
    print("Each line is a separate item that can be randomly indexed.")


if __name__ == "__main__":
    main()
