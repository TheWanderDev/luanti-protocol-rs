#!/usr/bin/env python3

import fcntl
import json
import posixpath
import re
import shutil
import subprocess
import sys
from collections import defaultdict
from html.parser import HTMLParser
from pathlib import Path

CRATE_NAME = "luanti_protocol_rs"
CRATE_PACKAGE = "luanti-protocol-rs"
API_DIRECTORY = Path("api") / CRATE_NAME
INLINE_CODE_PATTERN = re.compile(r"(?<!`)`([^`\n]+)`(?!`)")
FENCE_PATTERN = re.compile(r"^ {0,3}(`{3,}|~{3,})")
IDENTIFIER_PATTERN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")

KIND_PREFIXES = {
    "struct": "struct",
    "enum": "enum",
    "union": "union",
    "trait": "trait",
    "type": "type",
    "function": "fn",
    "proc_attribute": "attr",
    "proc_derive": "derive",
    "macro": "macro",
    "constant": "constant",
    "static": "static",
    "primitive": "primitive",
    "keyword": "keyword",
}

ANCHOR_KINDS = {
    "function": ("method", "tymethod"),
    "method": ("method", "tymethod"),
    "assoc_const": ("associatedconstant",),
    "assoc_type": ("associatedtype",),
    "struct_field": ("structfield",),
    "variant": ("variant",),
}


class AnchorParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        element_id = dict(attrs).get("id")
        if element_id:
            self.ids.add(element_id)


class RustdocIndex:
    def __init__(self, rustdoc_root, crate_name, crate_data):
        self.rustdoc_root = rustdoc_root
        self.crate_name = crate_name
        self.crate_data = crate_data
        self.items = crate_data["index"]
        self.paths = crate_data["paths"]
        self.targets = {}
        self.simple_names = defaultdict(set)
        self.anchor_cache = {}
        self.item_paths = {}
        self.item_kinds = {}

    @staticmethod
    def normalize_path(value):
        parts = [part.strip() for part in value.strip().split("::") if part.strip()]
        while parts and parts[0] in {"crate", "self", CRATE_NAME}:
            parts.pop(0)
        return "::".join(parts)

    @staticmethod
    def item_inner(item):
        inner = item.get("inner")
        if not isinstance(inner, dict) or len(inner) != 1:
            return None, None
        kind, value = next(iter(inner.items()))
        return kind, value

    @staticmethod
    def path_parts(path):
        if not isinstance(path, list):
            return ()
        parts = tuple(part for part in path if isinstance(part, str) and part)
        if parts and parts[0] in {"crate", CRATE_NAME}:
            return parts[1:]
        return parts

    def page_for(self, path_parts, kind):
        if kind == "module":
            relative = Path(*path_parts, "index.html") if path_parts else Path("index.html")
        else:
            if not path_parts:
                return None
            prefix = KIND_PREFIXES.get(kind)
            if prefix is None:
                return None
            parent = Path(*path_parts[:-1])
            relative = parent / f"{prefix}.{path_parts[-1]}.html"

        candidate = self.rustdoc_root / relative
        if candidate.is_file():
            return relative.as_posix()
        return None

    def add_target(self, path_parts, page, anchor=None):
        if not path_parts or not page:
            return
        key = "::".join(path_parts)
        self.targets.setdefault(key, (page, anchor))
        self.simple_names[path_parts[-1]].add(key)

    def load_anchors(self, page):
        cached = self.anchor_cache.get(page)
        if cached is not None:
            return cached

        parser = AnchorParser()
        try:
            with (self.rustdoc_root / page).open("r", encoding="utf-8") as source:
                parser.feed(source.read())
        except (OSError, UnicodeError):
            parser.close()
            self.anchor_cache[page] = set()
            return self.anchor_cache[page]

        parser.close()
        self.anchor_cache[page] = parser.ids
        return parser.ids

    def add_associated(self, parent_id, parent_path, parent_page):
        parent_item = self.items.get(str(parent_id))
        if not isinstance(parent_item, dict):
            return

        kind, data = self.item_inner(parent_item)
        child_ids = []

        if kind in {"struct", "union"} and isinstance(data, dict):
            shape = data.get("kind")
            if isinstance(shape, dict):
                plain = shape.get("plain")
                if isinstance(plain, dict):
                    child_ids.extend(plain.get("fields", []))
                tuple_fields = shape.get("tuple")
                if isinstance(tuple_fields, list):
                    child_ids.extend(field_id for field_id in tuple_fields if field_id)
            child_ids.extend(data.get("impls", []))
        elif kind == "enum" and isinstance(data, dict):
            child_ids.extend(data.get("variants", []))
            child_ids.extend(data.get("impls", []))
        elif kind == "trait" and isinstance(data, dict):
            child_ids.extend(data.get("items", []))

        for child_id in child_ids:
            self.add_child_item(child_id, parent_path, parent_page)

    def add_child_item(self, child_id, parent_path, parent_page):
        child = self.items.get(str(child_id))
        if not isinstance(child, dict):
            return

        kind, data = self.item_inner(child)
        if kind == "impl":
            if isinstance(data, dict):
                for impl_item_id in data.get("items", []):
                    self.add_child_item(impl_item_id, parent_path, parent_page)
            return

        name = child.get("name")
        if not isinstance(name, str) or not name:
            return

        anchor_kinds = ANCHOR_KINDS.get(kind, ())
        if not anchor_kinds:
            return

        anchors = self.load_anchors(parent_page)
        anchor = next(
            (f"{anchor_kind}.{name}" for anchor_kind in anchor_kinds
             if f"{anchor_kind}.{name}" in anchors),
            None,
        )
        if anchor:
            self.add_target((*parent_path, name), parent_page, anchor)

    def build(self):
        if not isinstance(self.items, dict) or not isinstance(self.paths, dict):
            raise ValueError("Struktur Rustdoc JSON tidak valid: index/paths harus object")

        for item_id, summary in self.paths.items():
            if not isinstance(summary, dict):
                continue
            path_parts = self.path_parts(summary.get("path"))
            kind = summary.get("kind")
            if not path_parts or not isinstance(kind, str):
                continue
            page = self.page_for(path_parts, kind)
            if page:
                self.add_target(path_parts, page)
                self.item_paths[str(item_id)] = path_parts
                self.item_kinds[str(item_id)] = kind

        for item_id, path_parts in tuple(self.item_paths.items()):
            kind = self.item_kinds.get(item_id)
            if kind not in {"struct", "union", "enum", "trait"}:
                continue
            page = self.targets.get("::".join(path_parts))
            if page:
                self.add_associated(item_id, path_parts, page[0])

        self.add_reexports()
        return self

    def add_reexports(self):
        root_id = self.crate_data.get("root")
        root_item = self.items.get(str(root_id))
        if not isinstance(root_item, dict):
            return

        visited = set()
        self.add_module_reexports(root_id, (), visited)

    def add_module_reexports(self, module_id, module_path, visited):
        visit_key = (str(module_id), module_path)
        if visit_key in visited:
            return
        visited.add(visit_key)

        module_item = self.items.get(str(module_id))
        if not isinstance(module_item, dict):
            return
        kind, module_data = self.item_inner(module_item)
        if kind != "module" or not isinstance(module_data, dict):
            return

        for child_id in module_data.get("items", []):
            child = self.items.get(str(child_id))
            if not isinstance(child, dict):
                continue

            child_kind, child_data = self.item_inner(child)
            if child_kind == "use" and isinstance(child_data, dict):
                target_id = child_data.get("id")
                alias_name = child_data.get("name") or child.get("name")
                if target_id is None or not isinstance(alias_name, str):
                    continue
                alias_path = (*module_path, alias_name)
                target_path = self.item_paths.get(str(target_id))
                target = self.targets.get("::".join(target_path or ()))
                if target:
                    self.add_target(alias_path, target[0], target[1])
                target_item = self.items.get(str(target_id))
                if isinstance(target_item, dict):
                    target_kind, _ = self.item_inner(target_item)
                    if target_kind in {"struct", "union", "enum", "trait"} and target:
                        self.add_associated(target_id, alias_path, target[0])
                    elif target_kind == "module":
                        self.add_module_reexports(target_id, alias_path, visited)
                continue

            child_path = self.item_paths.get(str(child_id))
            if child_kind == "module" and child_path:
                self.add_module_reexports(child_id, child_path, visited)

    def resolve(self, expression):
        value = expression.strip()
        value = re.sub(r"\([^()]*\)$", "", value).strip()
        value = re.sub(r"<.*>$", "", value).strip()
        normalized = self.normalize_path(value)
        target = self.targets.get(normalized)
        if target:
            return target

        if "::" in normalized:
            suffix = f"::{normalized}"
            matches = [
                path for path in self.targets
                if path.endswith(suffix)
            ]
            if len(matches) == 1:
                return self.targets[matches[0]]
        elif IDENTIFIER_PATTERN.fullmatch(normalized):
            matches = self.simple_names.get(normalized, set())
            if len(matches) == 1:
                return self.targets[next(iter(matches))]
        return None

    def link(self, expression, chapter_path):
        target = self.resolve(expression)
        if target is None:
            return None
        page, anchor = target
        destination = (API_DIRECTORY / Path(page)).as_posix()
        relative = posixpath.relpath(
            destination,
            posixpath.dirname(chapter_path) or ".",
        )
        return f"{relative}#{anchor}" if anchor else relative


def find_manifest(book_root):
    for parent in (book_root, *book_root.parents):
        manifest = parent / "Cargo.toml"
        if manifest.is_file():
            try:
                metadata = subprocess.run(
                    ["cargo", "metadata", "--no-deps", "--format-version", "1",
                     "--manifest-path", str(manifest)],
                    check=True,
                    capture_output=True,
                    text=True,
                )
                data = json.loads(metadata.stdout)
            except (OSError, subprocess.CalledProcessError, json.JSONDecodeError):
                continue
            packages = data.get("packages", [])
            selected_package = next(
                (package for package in packages if package.get("name") == CRATE_PACKAGE),
                None,
            )
            if selected_package is not None:
                return manifest.resolve(), Path(data["target_directory"]), selected_package
    raise FileNotFoundError(f"Cargo.toml package {CRATE_PACKAGE} tidak ditemukan")


def generate_rustdoc_artifacts(manifest, package, target_directory):
    common_args = [
        "--manifest-path", str(manifest),
        "--package", package["name"],
        "--lib",
        "--target-dir", str(target_directory),
    ]
    crate_name = next(
        (target.get("name") for target in package.get("targets", [])
         if "lib" in target.get("kind", [])),
        CRATE_NAME,
    )
    json_path = target_directory / "doc" / f"{crate_name}.json"
    html_root = target_directory / "doc" / crate_name

    subprocess.run(
        [
            "cargo", "+nightly", "rustdoc", *common_args,
            "--", "-Z", "unstable-options", "--output-format", "json",
        ],
        check=True,
    )
    subprocess.run(
        ["cargo", "+nightly", "doc", *common_args, "--no-deps"],
        check=True,
    )

    if not (html_root / "index.html").is_file():
        subprocess.run(
            [
                "cargo", "+nightly", "clean",
                "--manifest-path", str(manifest),
                "--target-dir", str(target_directory),
                "--doc",
            ],
            check=True,
        )
        subprocess.run(
            ["cargo", "+nightly", "doc", *common_args, "--no-deps"],
            check=True,
        )
        subprocess.run(
            [
                "cargo", "+nightly", "rustdoc", *common_args,
                "--", "-Z", "unstable-options", "--output-format", "json",
            ],
            check=True,
        )

    if not json_path.is_file():
        raise FileNotFoundError(f"Rustdoc JSON tidak ditemukan: {json_path}")
    if not (html_root / "index.html").is_file():
        raise FileNotFoundError(f"Rustdoc HTML tidak ditemukan: {html_root / 'index.html'}")

    return json_path, html_root, crate_name


def load_rustdoc_json(json_path):
    with json_path.open("r", encoding="utf-8") as source:
        data = json.load(source)
    if not isinstance(data, dict):
        raise ValueError("Root Rustdoc JSON bukan object")
    version = data.get("format_version")
    if not isinstance(version, int):
        raise ValueError("Rustdoc JSON tidak memiliki format_version integer")
    if "root" not in data or data["root"] is None:
        raise ValueError("Rustdoc JSON tidak memiliki root item ID")
    if not isinstance(data.get("index"), dict) or not isinstance(data.get("paths"), dict):
        raise ValueError("Rustdoc JSON tidak memiliki index/paths yang valid")
    return data


def prepare_api_directory(book_root, rustdoc_root):
    destination = book_root / "src" / API_DIRECTORY
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(rustdoc_root, destination, dirs_exist_ok=True)
    return destination


def convert_inline_code(content, chapter_path, index, stats):
    def replace(match):
        original = match.group(1)
        target = index.link(original, chapter_path)
        if target is None:
            if "::" in original or original.startswith((CRATE_NAME, "crate::")):
                stats["unresolved"].add(original)
            return match.group(0)
        stats["resolved"] += 1
        return f"[`{original}`]({target})"

    return INLINE_CODE_PATTERN.sub(replace, content)


def convert_markdown(content, chapter_path, index, stats):
    output = []
    fence_character = None
    fence_length = 0

    for line in content.splitlines(keepends=True):
        match = FENCE_PATTERN.match(line)
        if match:
            fence = match.group(1)
            if fence_character is None:
                fence_character = fence[0]
                fence_length = len(fence)
            elif fence[0] == fence_character and len(fence) >= fence_length:
                fence_character = None
                fence_length = 0
            output.append(line)
            continue
        if fence_character is not None:
            output.append(line)
            continue
        output.append(convert_inline_code(line, chapter_path, index, stats))

    return "".join(output)


def process_chapter(chapter, index, stats):
    if not isinstance(chapter, dict):
        return
    content = chapter.get("content")
    if isinstance(content, str):
        chapter_path = chapter.get("path")
        if not isinstance(chapter_path, str) or not chapter_path:
            chapter_path = "index.md"
        chapter["content"] = convert_markdown(content, chapter_path, index, stats)
        stats["chapters"] += 1
    children = chapter.get("sub_items", [])
    if isinstance(children, list):
        for child in children:
            process_book_item(child, index, stats)


def process_book_item(item, index, stats):
    if not isinstance(item, dict):
        return
    chapter = item.get("Chapter")
    if isinstance(chapter, dict):
        process_chapter(chapter, index, stats)
    elif isinstance(item.get("content"), str):
        process_chapter(item, index, stats)


def process_book(book, index, stats):
    if not isinstance(book, dict):
        raise ValueError("Struktur Book mdBook tidak valid")
    items = book.get("items")
    if not isinstance(items, list):
        items = book.get("sections")
    if not isinstance(items, list):
        raise ValueError("Book mdBook tidak memiliki items/sections")
    for item in items:
        process_book_item(item, index, stats)
    if stats["chapters"] == 0:
        raise RuntimeError("Tidak ada chapter Markdown yang berhasil diproses")


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "supports":
        return 0

    payload = json.load(sys.stdin)
    if not isinstance(payload, list) or len(payload) != 2:
        raise ValueError("Input mdBook harus berupa pasangan context dan book")
    context, book = payload
    if not isinstance(context, dict):
        raise ValueError("Context mdBook tidak valid")
    root_value = context.get("root")
    if not isinstance(root_value, str) or not root_value:
        raise ValueError("Root directory mdBook tidak tersedia")

    book_root = Path(root_value).resolve()
    manifest, target_directory, package = find_manifest(book_root)
    target_directory.mkdir(parents=True, exist_ok=True)
    lock_path = target_directory / ".rustdoc-link-resolver.lock"

    with lock_path.open("a", encoding="utf-8") as lock_file:
        fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX)
        json_path, rustdoc_root, crate_name = generate_rustdoc_artifacts(
            manifest,
            package,
            target_directory,
        )
        crate_data = load_rustdoc_json(json_path)
        index = RustdocIndex(rustdoc_root, crate_name, crate_data).build()
        prepare_api_directory(book_root, rustdoc_root)
        fcntl.flock(lock_file.fileno(), fcntl.LOCK_UN)

    stats = {"chapters": 0, "resolved": 0, "unresolved": set()}
    process_book(book, index, stats)

    print(
        f"Rustdoc JSON format: {crate_data['format_version']}; "
        f"indexed paths: {len(index.targets)}; "
        f"chapters: {stats['chapters']}; "
        f"resolved: {stats['resolved']}; "
        f"unresolved unique paths: {len(stats['unresolved'])}",
        file=sys.stderr,
    )
    for path in sorted(stats["unresolved"]):
        print(f"rustdoc-link-resolver: unresolved: {path}", file=sys.stderr)

    json.dump(book, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (
        OSError,
        ValueError,
        RuntimeError,
        KeyError,
        json.JSONDecodeError,
        subprocess.CalledProcessError,
    ) as error:
        print(f"rustdoc-link-resolver: {error}", file=sys.stderr)
        sys.exit(1)
