#this python code not intended to meet production ready standards but it is intended to quickly resolve rustdoc link issue during development
#!/usr/bin/env python3
"""Resolve Rustdoc-style Markdown links against generated Rustdoc HTML."""

import json
import posixpath
import re
import sys
from collections import defaultdict
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath
from urllib.parse import unquote, urlsplit

CRATE_NAMES = ("luanti_protocol_rs", "luanti-protocol-rs")

ITEM_PREFIXES = (
    "struct",
    "enum",
    "trait",
    "union",
    "fn",
    "type",
    "constant",
    "static",
    "macro",
    "primitive",
    "attr",
    "derive",
    "proc",
    "mod",
)

ASSOCIATED_PREFIXES = (
    "method",
    "tymethod",
    "associatedtype",
    "associatedconstant",
    "variant",
    "structfield",
)

NORMALISE_PREFIXES = tuple(f"{kind}@" for kind in ITEM_PREFIXES)

RUSTDOC_LINK = re.compile(r"(?<!!)\[`([^`\n]+)`\]")
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})")

VOID_ELEMENTS = frozenset(
    {
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta",
        "param",
        "source",
        "track",
        "wbr",
    }
)


class AnchorCollector(HTMLParser):
    """Collect HTML links and element IDs."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.links: list[tuple[str, str]] = []
        self.ids: list[str] = []
        self._href: str | None = None
        self._text: list[str] = []
        self._depth = 0

    def handle_starttag(
        self,
        tag: str,
        attrs: list[tuple[str, str | None]],
    ) -> None:
        values = dict(attrs)

        element_id = values.get("id")
        if element_id is not None:
            self.ids.append(element_id)

        href = values.get("href")

        if tag == "a" and href is not None:
            self._href = href
            self._text = []
            self._depth = 1
        elif self._depth and tag not in VOID_ELEMENTS:
            self._depth += 1

    def handle_startendtag(
        self,
        tag: str,
        attrs: list[tuple[str, str | None]],
    ) -> None:
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag: str) -> None:
        if not self._depth:
            return

        if tag == "a" and self._depth == 1:
            if self._href is not None:
                self.links.append(
                    (self._href, "".join(self._text).strip())
                )

            self._href = None
            self._text = []
            self._depth = 0
        elif tag not in VOID_ELEMENTS:
            self._depth = max(1, self._depth - 1)

    def handle_data(self, data: str) -> None:
        if self._depth:
            self._text.append(data)


class RustdocIndex:
    """Index Rustdoc pages, public paths, aliases and associated items."""

    def __init__(self, api_root: Path) -> None:
        self.api_root = api_root
        self.targets: dict[str, str] = {}
        self.page_aliases: dict[str, set[str]] = defaultdict(set)
        self.html_files: list[Path] = []
        self._load()

    @staticmethod
    def _item_path(
        relative: PurePosixPath,
    ) -> tuple[str, str] | None:
        name = relative.name

        if name == "index.html":
            return "::".join(relative.parent.parts), ""

        if not name.endswith(".html"):
            return None

        stem = name[:-5]

        for prefix in ITEM_PREFIXES:
            marker = f"{prefix}."

            if stem.startswith(marker):
                item_name = stem[len(marker):]
                module = "::".join(relative.parent.parts)

                item_path = "::".join(
                    part for part in (module, item_name) if part
                )

                return item_path, prefix

        return None

    @staticmethod
    def _normalise(rust_path: str) -> str:
        value = rust_path.strip().strip("`").replace(" ", "")

        for prefix in NORMALISE_PREFIXES:
            if value.startswith(prefix):
                value = value[len(prefix):]
                break

        for crate_name in CRATE_NAMES:
            marker = f"{crate_name}::"

            if value.startswith(marker):
                value = value[len(marker):]
                break

        if value.startswith("crate::"):
            value = value[len("crate::"):]

        return value.removeprefix("::")

    def _insert(self, rust_path: str, target: str) -> None:
        key = self._normalise(rust_path)

        if key:
            self.targets.setdefault(key, target)

    @staticmethod
    def _read_html(html_file: Path) -> AnchorCollector:
        parser = AnchorCollector()
        parser.feed(
            html_file.read_text(
                encoding="utf-8",
                errors="replace",
            )
        )
        parser.close()
        return parser

    @staticmethod
    def _book_url(target_file: PurePosixPath) -> str:
        return (
            PurePosixPath("api/luanti_protocol_rs") / target_file
        ).as_posix()

    @staticmethod
    def _associated_name(fragment: str) -> str | None:
        for prefix in ASSOCIATED_PREFIXES:
            marker = f"{prefix}."

            if fragment.startswith(marker):
                return fragment[len(marker):]

        return None

    @staticmethod
    def _item_name(fragment: str) -> str | None:
        for prefix in ITEM_PREFIXES:
            marker = f"{prefix}."

            if fragment.startswith(marker):
                return fragment[len(marker):]

        return None

    def _insert_associated(
        self,
        target_file: PurePosixPath,
        target_path: str,
        fragment: str,
    ) -> None:
        associated_name = self._associated_name(fragment)

        if associated_name is None:
            return

        page_url = self._book_url(target_file)
        target_url = f"{page_url}#{fragment}"

        self._insert(
            f"{target_path}::{associated_name}",
            target_url,
        )

        for alias in self.page_aliases.get(target_file.as_posix(), ()):
            self._insert(
                f"{alias}::{associated_name}",
                target_url,
            )

    def _insert_item_fragment(
        self,
        target_path: str,
        target_file: PurePosixPath,
        fragment: str,
    ) -> None:
        item_name = self._item_name(fragment)

        if item_name is None:
            return

        self._insert(
            f"{target_path}::{item_name}",
            f"{self._book_url(target_file)}#{fragment}",
        )

    def _resolve_target_file(
        self,
        relative: PurePosixPath,
        href: str,
    ) -> PurePosixPath | None:
        parsed = urlsplit(href)

        if parsed.scheme or parsed.netloc:
            return None

        if not parsed.path:
            return relative

        target_path = posixpath.normpath(
            posixpath.join(
                relative.parent.as_posix(),
                unquote(parsed.path),
            )
        )

        target_file = PurePosixPath(target_path)

        if target_file.is_absolute() or ".." in target_file.parts:
            return None

        return target_file

    def _load_pages(self) -> None:
        root_index = self.api_root / "index.html"

        if not root_index.is_file():
            raise FileNotFoundError(
                f"Rustdoc HTML tidak ditemukan: {root_index}"
            )

        self.html_files = sorted(self.api_root.rglob("*.html"))

        for html_file in self.html_files:
            relative = PurePosixPath(
                html_file.relative_to(self.api_root).as_posix()
            )

            page = self._item_path(relative)

            if page is None:
                continue

            current_path, current_kind = page

            if current_path and current_kind:
                self._insert(
                    current_path,
                    self._book_url(relative),
                )

    def _load_page_links(self) -> None:
        for html_file in self.html_files:
            relative = PurePosixPath(
                html_file.relative_to(self.api_root).as_posix()
            )

            page = self._item_path(relative)

            if page is None:
                continue

            current_path, current_kind = page
            parser = self._read_html(html_file)

            for href, label in parser.links:
                target_file = self._resolve_target_file(
                    relative,
                    href,
                )

                if target_file is None:
                    continue

                target_page = self._item_path(target_file)

                if target_page is None:
                    continue

                target_path, target_kind = target_page
                target_url = self._book_url(target_file)

                if target_path:
                    self._insert(target_path, target_url)

                if label and not current_kind:
                    alias_path = "::".join(
                        part for part in (current_path, label) if part
                    )

                    self._insert(alias_path, target_url)

                    self.page_aliases[target_file.as_posix()].add(
                        alias_path
                    )

    def _load_page_fragments(self) -> None:
        for html_file in self.html_files:
            relative = PurePosixPath(
                html_file.relative_to(self.api_root).as_posix()
            )

            page = self._item_path(relative)

            if page is None:
                continue

            current_path, current_kind = page
            parser = self._read_html(html_file)

            for element_id in parser.ids:
                if current_kind:
                    self._insert_associated(
                        relative,
                        current_path,
                        element_id,
                    )
                else:
                    self._insert_item_fragment(
                        current_path,
                        relative,
                        element_id,
                    )

            for href, _ in parser.links:
                parsed = urlsplit(href)

                if not parsed.fragment:
                    continue

                target_file = self._resolve_target_file(
                    relative,
                    href,
                )

                if target_file is None:
                    continue

                target_page = self._item_path(target_file)

                if target_page is None:
                    continue

                target_path, target_kind = target_page
                fragment = unquote(parsed.fragment)

                if target_kind:
                    self._insert_associated(
                        target_file,
                        target_path,
                        fragment,
                    )
                else:
                    self._insert_item_fragment(
                        target_path,
                        target_file,
                        fragment,
                    )

    def _load(self) -> None:
        self._load_pages()

        # Re-export aliases are collected before associated item indexing.
        self._load_page_links()

        # Enum variants and other associated items use those aliases.
        self._load_page_fragments()

    def resolve(self, rust_path: str) -> str | None:
        key = self._normalise(rust_path)
        return self.targets.get(key)


def _convert_segment(
    segment: str,
    index: RustdocIndex,
    chapter_output: str,
) -> tuple[str, list[str]]:
    output: list[str] = []
    unresolved: list[str] = []
    cursor = 0

    while cursor < len(segment):
        if segment[cursor] == "[":
            match = RUSTDOC_LINK.match(segment, cursor)

            if match is not None:
                end = match.end()

                if end == len(segment) or segment[end] not in "([":
                    rust_path = match.group(1)
                    target = index.resolve(rust_path)

                    if target is None:
                        if rust_path.startswith(
                            (
                                "crate::",
                                "luanti_protocol_rs::",
                                "luanti-protocol-rs::",
                            )
                        ):
                            unresolved.append(rust_path)

                        output.append(match.group(0))
                    else:
                        relative_url = posixpath.relpath(
                            target,
                            posixpath.dirname(chapter_output) or ".",
                        )

                        output.append(
                            f"[`{rust_path}`]({relative_url})"
                        )

                    cursor = end
                    continue

        if segment[cursor] == "`":
            run_end = cursor

            while (
                run_end < len(segment)
                and segment[run_end] == "`"
            ):
                run_end += 1

            marker = segment[cursor:run_end]
            closing = segment.find(marker, run_end)

            if closing >= 0:
                output.append(
                    segment[cursor:closing + len(marker)]
                )
                cursor = closing + len(marker)
                continue

        output.append(segment[cursor])
        cursor += 1

    return "".join(output), unresolved


def _convert_markdown(
    content: str,
    index: RustdocIndex,
    chapter_output: str,
) -> tuple[str, list[str]]:
    output: list[str] = []
    unresolved: list[str] = []
    fence_char: str | None = None
    fence_size = 0

    for line in content.splitlines(keepends=True):
        fence_match = FENCE.match(line)

        if fence_char is not None:
            output.append(line)

            if (
                fence_match
                and fence_match.group(1)[0] == fence_char
                and len(fence_match.group(1)) >= fence_size
            ):
                fence_char = None
                fence_size = 0

            continue

        if fence_match:
            fence = fence_match.group(1)
            fence_char = fence[0]
            fence_size = len(fence)
            output.append(line)
            continue

        if line.startswith(("    ", "\t")):
            output.append(line)
            continue

        converted, missing = _convert_segment(
            line,
            index,
            chapter_output,
        )

        output.append(converted)
        unresolved.extend(missing)

    return "".join(output), unresolved


def _chapter_output_path(
    chapter: dict[str, object],
) -> str | None:
    path_value = chapter.get("path")

    if isinstance(path_value, str) and path_value:
        return PurePosixPath(path_value).as_posix()

    source_value = chapter.get("source_path")

    if isinstance(source_value, str) and source_value:
        return PurePosixPath(source_value).with_suffix(".html").as_posix()

    return None


def _walk_chapters(
    items: list[object],
    index: RustdocIndex,
) -> list[str]:
    unresolved: list[str] = []

    for item in items:
        if not isinstance(item, dict):
            continue

        chapter = item.get("Chapter")

        if not isinstance(chapter, dict):
            continue

        output_path = _chapter_output_path(chapter)
        content = chapter.get("content")

        if output_path is not None and isinstance(content, str):
            converted, missing = _convert_markdown(
                content,
                index,
                output_path,
            )

            chapter["content"] = converted
            unresolved.extend(missing)

        children = chapter.get("sub_items")

        if isinstance(children, list):
            unresolved.extend(
                _walk_chapters(children, index)
            )

    return unresolved


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "supports":
        return int(
            len(sys.argv) > 2 and sys.argv[2] == "html"
        ) ^ 1

    try:
        payload = json.load(sys.stdin)

        if not isinstance(payload, list) or len(payload) != 2:
            raise ValueError(
                "Input mdBook harus berupa pasangan [context, book]."
            )

        context, book = payload

        if not isinstance(context, dict) or not isinstance(book, dict):
            raise ValueError("Struktur input mdBook tidak valid.")

        root_value = context.get("root")

        if not isinstance(root_value, str):
            raise ValueError(
                "mdBook tidak memberikan context.root."
            )

        root = Path(root_value)
        api_root = root / "src" / "api" / "luanti_protocol_rs"
        index = RustdocIndex(api_root)

        items = book.get("sections")

        if not isinstance(items, list):
            items = book.get("items")

        if not isinstance(items, list):
            raise ValueError(
                "Daftar chapter mdBook tidak ditemukan."
            )

        unresolved = _walk_chapters(items, index)

        if unresolved:
            unique = sorted(set(unresolved))

            print(
                f"rustdoc-link-resolver: {len(unique)} "
                "referensi crate belum ditemukan:",
                file=sys.stderr,
            )

            for rust_path in unique:
                print(f"  {rust_path}", file=sys.stderr)

        sys.stdout.write(json.dumps(book))
        sys.stdout.write("\n")

        return 0

    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(
            f"rustdoc-link-resolver: {error}",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
