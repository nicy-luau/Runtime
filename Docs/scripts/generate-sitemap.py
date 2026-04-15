#!/usr/bin/env python3
from __future__ import annotations

import argparse
import datetime as dt
from pathlib import Path
import xml.etree.ElementTree as ET

SITE_ROOT = "https://nicy-luau.github.io/Runtime/"
XMLNS = "http://www.sitemaps.org/schemas/sitemap/0.9"
EXCLUDED_FILES = {"print.html", "404.html"}


def iter_html_files(book_dir: Path) -> list[Path]:
    return sorted(
        path
        for path in book_dir.rglob("*.html")
        if path.is_file() and path.name not in EXCLUDED_FILES
    )


def to_absolute_url(book_dir: Path, html_file: Path, site_root: str) -> str:
    rel_path = html_file.relative_to(book_dir).as_posix()
    if rel_path == "index.html":
        return site_root
    if rel_path.endswith("/index.html"):
        return f"{site_root}{rel_path[:-len('index.html')]}"
    return f"{site_root}{rel_path[:-len('.html')]}/"


def metadata_for(url: str, site_root: str) -> tuple[str, str]:
    if url == site_root:
        return "1.0", "weekly"
    if "/getting-started/" in url:
        return "0.9", "weekly"
    if "/ffi-reference/" in url or "/runtime-api/" in url:
        return "0.8", "weekly"
    return "0.7", "monthly"


def build_sitemap(book_dir: Path, site_root: str, lastmod: str) -> tuple[ET.ElementTree, int]:
    ET.register_namespace("", XMLNS)
    root = ET.Element(f"{{{XMLNS}}}urlset")

    count = 0
    for html_file in iter_html_files(book_dir):
        url = to_absolute_url(book_dir, html_file, site_root)
        priority, changefreq = metadata_for(url, site_root)

        url_el = ET.SubElement(root, f"{{{XMLNS}}}url")
        ET.SubElement(url_el, f"{{{XMLNS}}}loc").text = url
        ET.SubElement(url_el, f"{{{XMLNS}}}lastmod").text = lastmod
        ET.SubElement(url_el, f"{{{XMLNS}}}priority").text = priority
        ET.SubElement(url_el, f"{{{XMLNS}}}changefreq").text = changefreq
        count += 1

    tree = ET.ElementTree(root)
    ET.indent(tree, space="  ")
    return tree, count


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate sitemap.xml for the built mdBook site.")
    parser.add_argument("--book-dir", default="Docs/book", help="Path to the built mdBook output directory.")
    parser.add_argument("--site-root", default=SITE_ROOT, help="Absolute site root URL.")
    parser.add_argument(
        "--lastmod",
        default=dt.date.today().isoformat(),
        help="ISO date to use as <lastmod> for every URL.",
    )
    args = parser.parse_args()

    book_dir = Path(args.book_dir).resolve()
    if not book_dir.is_dir():
        raise SystemExit(f"Book directory not found: {book_dir}")

    site_root = args.site_root.rstrip("/") + "/"
    sitemap_path = book_dir / "sitemap.xml"

    tree, count = build_sitemap(book_dir, site_root, args.lastmod)
    tree.write(sitemap_path, encoding="utf-8", xml_declaration=True)

    print(f"Generated {sitemap_path} with {count} URLs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
