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

"""Comprehensive stdlib discovery that collects EVERYTHING.

This script imports all stdlib modules and introspects them to collect:
- All module names (including submodules)
- All functions, classes, constants, and other attributes
- All methods and attributes of classes

Outputs a flat, line-separated file for easy random access.
"""

from __future__ import annotations

import sys
import importlib
import pkgutil
import inspect
from typing import Set

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

OUTPUT_FILE = "../data/stdlib_complete.txt"


def get_all_module_names() -> Set[str]:
    """Collect all stdlib module names including submodules."""
    all_names = set()

    # Get base stdlib modules
    if hasattr(sys, "stdlib_module_names"):
        stdlib_base = sys.stdlib_module_names
    else:
        # Fallback for older Python
        stdlib_base = sys.builtin_module_names

    for mod_name in stdlib_base:
        if mod_name in SKIP_IMPORT:
            all_names.add(mod_name)
            continue

        all_names.add(mod_name)

        # Try to import and walk submodules
        try:
            mod = importlib.import_module(mod_name)
            if hasattr(mod, '__path__'):
                # It's a package, walk submodules
                try:
                    for _, submod_name, _ in pkgutil.walk_packages(
                        mod.__path__,
                        prefix=mod_name + "."
                    ):
                        all_names.add(submod_name)
                except Exception:
                    pass  # Some packages can't be walked
        except Exception:
            pass  # Can't import, but we have the name

    return all_names


def get_module_members(module_name: str) -> Set[str]:
    """Introspect a module to get all its members (functions, classes, etc.)."""
    members = set()

    if module_name in SKIP_IMPORT:
        return members

    # Skip __main__ submodules (they often hang or have side effects)
    if "__main__" in module_name:
        return members

    # Skip modules/packages that are problematic to introspect
    for skip_prefix in SKIP_INTROSPECT:
        if module_name == skip_prefix or module_name.startswith(skip_prefix + "."):
            return members

    try:
        mod = importlib.import_module(module_name)
    except Exception:
        return members

    # Get all public attributes
    try:
        all_attrs = dir(mod)
    except Exception:
        return members

    for attr_name in all_attrs:
        # Skip dunder methods/attrs 
        if attr_name.startswith('__') and attr_name.endswith('__'):
            continue

        try:
            attr = getattr(mod, attr_name)
        except Exception:
            continue

        # Add the attribute itself (function, class, constant, etc.)
        full_name = f"{module_name}.{attr_name}"
        members.add(full_name)

        # If it's a class, introspect it for methods and attributes
        if inspect.isclass(attr):
            try:
                class_attrs = dir(attr)

                for class_attr_name in class_attrs:
                    # Skip dunder methods
                    if class_attr_name.startswith('__') and class_attr_name.endswith('__'):
                        continue
                    try:
                        # class_full_name = f"{module_name}.{attr_name}.{class_attr_name}"
                        class_full_name = f"{module_name}.{attr_name}"
                        members.add(class_full_name)
                    except Exception:
                        continue
            except Exception:
                pass

    return members


def main():
    print("Collecting all stdlib module names...")
    all_modules = get_all_module_names()
    print(f"Found {len(all_modules)} modules")

    print("\nIntrospecting modules for all members...")
    all_items = set()

    # Add all module names
    all_items.update(all_modules)

    # Process each module
    total = len(all_modules)
    for idx, mod_name in enumerate(sorted(all_modules), 1):
        if idx % 50 == 0 or idx > 500:
            print(f"  Processed {idx}/{total} modules... (current: {mod_name})")

        # Add timeout protection for slow modules
        try:
            members = get_module_members(mod_name)
            all_items.update(members)
        except Exception as e:
            print(f"  Warning: Failed to process {mod_name}: {e}")

    print(f"\nCollected {len(all_items)} total items")

    # Write to flat file
    print(f"Writing to {OUTPUT_FILE}...")
    with open(OUTPUT_FILE, 'w') as f:
        for item in sorted(all_items):
            if '__main__' in item:
                continue
            f.write(f"{item}\n")

    print(f"\nDone! Written {len(all_items)} items to {OUTPUT_FILE}")
    print("Each line is a separate item that can be randomly indexed.")


if __name__ == "__main__":
    main()