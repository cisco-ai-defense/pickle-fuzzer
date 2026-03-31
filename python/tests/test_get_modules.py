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

import importlib.util
from pathlib import Path

import pytest


SCRIPT_PATH = Path(__file__).resolve().parents[2] / "scripts" / "get-modules.py"
SPEC = importlib.util.spec_from_file_location("get_modules", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
get_modules = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(get_modules)


def test_get_stdlib_base_module_names_requires_stdlib_module_names(monkeypatch):
    monkeypatch.delattr(get_modules.sys, "stdlib_module_names", raising=False)

    with pytest.raises(RuntimeError, match="sys.stdlib_module_names"):
        get_modules.get_stdlib_base_module_names()


def test_get_package_submodule_names_does_not_import_package_tree(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    marker = tmp_path / "imported.txt"
    package_dir = tmp_path / "dangerpkg"
    package_dir.mkdir()
    nested_dir = package_dir / "nested"
    nested_dir.mkdir()

    marker_literal = repr(str(marker))
    (package_dir / "__init__.py").write_text(
        f"from pathlib import Path\nPath({marker_literal}).write_text('root')\n",
        encoding="utf-8",
    )
    (package_dir / "child.py").write_text("VALUE = 1\n", encoding="utf-8")
    (nested_dir / "__init__.py").write_text(
        f"from pathlib import Path\nPath({marker_literal}).write_text('nested')\n",
        encoding="utf-8",
    )
    (nested_dir / "leaf.py").write_text("VALUE = 2\n", encoding="utf-8")

    monkeypatch.syspath_prepend(str(tmp_path))

    names = get_modules.get_package_submodule_names("dangerpkg")

    assert "dangerpkg.child" in names
    assert "dangerpkg.nested" in names
    assert "dangerpkg.nested.leaf" in names
    assert not marker.exists()


def test_get_module_members_emit_unambiguous_entries():
    members = get_modules.get_module_members("xml.etree.ElementTree")

    assert "xml.etree.ElementTree\tComment" in members
    assert members
    assert all("\t" in entry for entry in members)
