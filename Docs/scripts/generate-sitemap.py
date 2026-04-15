#!/usr/bin/env python3
"""
generate-sitemap.py — Automatic sitemap generator for NicyRuntime mdBook docs.

Scans Docs/book/**/*.html, extracts paths, and generates a valid sitemap.xml
with lastmod, priority, and changefreq for SEO optimization.

Usage:
    python generate-sitemap.py [--base-url URL] [--book-dir DIR] [--output FILE]

Example:
    python generate-sitemap.py --base-url https://nicy-luau.github.io/Runtime/
"""

import os
import re
import sys
import argparse
from datetime import datetime, timezone
from xml.etree import ElementTree as ET
from pathlib import Path


DEFAULT_BASE_URL = "https://nicy-luau.github.io/Runtime/"
DEFAULT_BOOK_DIR = os.path.join(os.path.dirname(__file__), "..", "book")
DEFAULT_OUTPUT = os.path.join(os.path.dirname(__file__), "..", "book", "sitemap.xml")

# Priority mapping by section
SECTION_PRIORITY = {
    "index.html": 1.0,
    "getting-started": 0.9,
    "cli": 0.8,
    "runtime-api": 0.85,
    "ffi-reference": 0.8,
    "guides": 0.75,
    "advanced": 0.7,
    "testing": 0.6,
    "troubleshooting": 0.5,
}

# Change frequency by section
SECTION_FREQ = {
    "index.html": "weekly",
    "getting-started": "monthly",
    "cli": "monthly",
    "runtime-api": "monthly",
    "ffi-reference": "monthly",
    "guides": "monthly",
    "advanced": "monthly",
    "testing": "monthly",
    "troubleshooting": "weekly",
}


def get_lastmod(filepath: str) -> str:
    """Get file modification date in W3C format (YYYY-MM-DD)."""
    mtime = os.path.getmtime(filepath)
    dt = datetime.fromtimestamp(mtime, tz=timezone.utc)
    return dt.strftime("%Y-%m-%d")


def extract_html_paths(book_dir: str) -> list[tuple[str, str]]:
    """
    Scan book directory for HTML files and extract (url, lastmod) tuples.
    
    Returns list of (relative_url, lastmod_date) tuples.
    """
    urls = []
    book_path = Path(book_dir)
    
    if not book_path.exists():
        print(f"Error: book directory '{book_dir}' does not exist.", file=sys.stderr)
        sys.exit(1)
    
    for html_file in book_path.rglob("*.html"):
        # Skip sitemap.xml itself and hidden files
        if html_file.name == "sitemap.xml":
            continue
        if html_file.name.startswith("."):
            continue
        
        # Get relative path from book root
        rel_path = html_file.relative_to(book_path)
        
        # Convert to URL
        if rel_path.name == "index.html":
            # Directory index: intro/index.html → /
            if rel_path.parent == Path("."):
                url_path = ""
            else:
                url_path = str(rel_path.parent) + "/"
        else:
            # Regular page: getting-started/installation.html → /getting-started/installation/
            url_path = str(rel_path.with_suffix("")) + "/"
        
        # Get file modification date
        lastmod = get_lastmod(str(html_file))
        
        urls.append((url_path, lastmod))
    
    return urls


def get_priority(url_path: str) -> str:
    """Determine priority based on URL section."""
    for section, priority in SECTION_PRIORITY.items():
        if url_path == "" and section == "index.html":
            return str(priority)
        if section in url_path:
            return str(priority)
    return "0.5"  # Default priority


def get_changefreq(url_path: str) -> str:
    """Determine change frequency based on URL section."""
    for section, freq in SECTION_FREQ.items():
        if url_path == "" and section == "index.html":
            return freq
        if section in url_path:
            return freq
    return "monthly"  # Default frequency


def generate_sitemap(urls: list[tuple[str, str]], base_url: str) -> str:
    """Generate sitemap XML string from list of (url, lastmod) tuples."""
    # XML namespace
    ns = "http://www.sitemaps.org/schemas/sitemap/0.9"
    ET.register_namespace("", ns)
    
    # Root element
    urlset = ET.Element("urlset", xmlns=ns)
    
    for url_path, lastmod in sorted(urls, key=lambda x: x[0]):
        # Build full URL
        full_url = base_url.rstrip("/") + "/" + url_path.lstrip("/")
        
        # Create <url> element
        url_elem = ET.SubElement(urlset, "url")
        
        # <loc> — required
        loc = ET.SubElement(url_elem, "loc")
        loc.text = full_url
        
        # <lastmod> — recommended
        lastmod_elem = ET.SubElement(url_elem, "lastmod")
        lastmod_elem.text = lastmod
        
        # <changefreq> — optional but recommended
        changefreq = ET.SubElement(url_elem, "changefreq")
        changefreq.text = get_changefreq(url_path)
        
        # <priority> — optional but recommended
        priority = ET.SubElement(url_elem, "priority")
        priority.text = get_priority(url_path)
    
    # Pretty print with XML declaration
    ET.indent(urlset, space="  ", level=0)
    xml_str = ET.tostring(urlset, encoding="unicode", xml_declaration=True)
    
    return xml_str


def validate_sitemap(xml_str: str) -> bool:
    """Validate that the generated XML is well-formed."""
    try:
        ET.fromstring(xml_str)
        return True
    except ET.ParseError as e:
        print(f"Error: Generated XML is invalid: {e}", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Generate sitemap.xml for NicyRuntime docs"
    )
    parser.add_argument(
        "--base-url",
        default=DEFAULT_BASE_URL,
        help=f"Base URL for sitemap (default: {DEFAULT_BASE_URL})"
    )
    parser.add_argument(
        "--book-dir",
        default=DEFAULT_BOOK_DIR,
        help=f"Path to mdBook output directory (default: {DEFAULT_BOOK_DIR})"
    )
    parser.add_argument(
        "--output",
        default=DEFAULT_OUTPUT,
        help=f"Output file path (default: {DEFAULT_OUTPUT})"
    )
    
    args = parser.parse_args()
    
    # Extract paths from HTML files
    print(f"Scanning {args.book_dir} for HTML files...")
    urls = extract_html_paths(args.book_dir)
    
    if len(urls) < 5:
        print(f"Warning: Only {len(urls)} URLs found. Expected at least 5.", file=sys.stderr)
        print("This may indicate a problem with the build.", file=sys.stderr)
        sys.exit(1)
    
    print(f"Found {len(urls)} pages.")
    
    # Generate sitemap XML
    xml_str = generate_sitemap(urls, args.base_url)
    
    # Validate XML
    if not validate_sitemap(xml_str):
        print("Error: Generated sitemap XML is invalid.", file=sys.stderr)
        sys.exit(1)
    
    # Write to file
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        f.write(xml_str)
    
    print(f"✓ Sitemap generated: {args.output}")
    print(f"  URLs: {len(urls)}")
    print(f"  Base URL: {args.base_url}")
    
    # Print preview of first 5 URLs
    print("\n  Preview (first 5 URLs):")
    for url_path, lastmod in sorted(urls, key=lambda x: x[0])[:5]:
        full_url = args.base_url.rstrip("/") + "/" + url_path.lstrip("/")
        priority = get_priority(url_path)
        print(f"    {full_url}")
        print(f"      lastmod={lastmod}, priority={priority}")


if __name__ == "__main__":
    main()
