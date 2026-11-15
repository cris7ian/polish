#!/usr/bin/env python3
"""Check the static site links and required accessibility metadata."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

root = Path(__file__).resolve().parent.parent / "website"

class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.ids = set()
        self.title = False
        self.viewport = False
        self.heading = False

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if "id" in a:
            self.ids.add(a["id"])
        if tag == "title":
            self.title = True
        if tag == "h1":
            self.heading = True
        if tag == "meta" and a.get("name") == "viewport":
            self.viewport = True
        for key in ("href", "src"):
            if key in a:
                self.links.append(a[key])
        if tag == "img" and "alt" not in a:
            raise ValueError("Image lacks alt text")

page = Page()
page.feed((root / "index.html").read_text())
assert page.title and page.viewport and page.heading
for link in page.links:
    parsed = urlparse(link)
    if parsed.scheme or parsed.netloc or link == "/":
        continue
    if parsed.path:
        assert (root / parsed.path.lstrip("/")).is_file(), f"Missing asset: {link}"
    if parsed.fragment:
        assert parsed.fragment in page.ids, f"Missing target: {link}"
print("Website checks passed")
