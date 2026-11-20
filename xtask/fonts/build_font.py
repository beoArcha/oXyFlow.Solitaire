"""Builds the oXySol Mono card font: the Cascadia Mono base plus the card glyph SVGs (D-06, T-16).

Run only by `cargo xtask fonts`, which checks the registry and passes one `--glyph KEY CODE_POINT SVG`
per registry key that has art. Python is needed for this command alone.

The SVG frame and the outline rules follow the style guide of M-19…M-24 (docs/task_list/micro.md).
"""

import argparse
import base64
import html
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

try:
    import pathops
    from fontTools.pens.boundsPen import BoundsPen
    from fontTools.pens.cu2quPen import Cu2QuPen
    from fontTools.pens.transformPen import TransformPen
    from fontTools.pens.ttGlyphPen import TTGlyphPen
    from fontTools.svgLib.path import parse_path
    from fontTools.ttLib import TTFont
except ImportError as missing:
    print(
        f"error: Python package `{missing.name}` is missing; install the pinned versions with "
        "`python -m pip install -r xtask/fonts/requirements.txt`",
        file=sys.stderr,
    )
    sys.exit(3)

UPM = 2048
ADVANCE = 1200
ASCENT = 1900
DESCENT = 480
CELL = (1200, 2380)
MOSAIC = (2400, 4760)
SAFE_AREA = (60, 60, 1140, 2320)
CODE_MIN = 0xE6BC
CODE_MAX = 0xE6FF
FAMILY = "oXySol Mono"
UNIQUE_ID = "oXySolMono-Regular;oXySol build"
COPYRIGHT_LINE = ' Card glyphs: Copyright (c) 2026 Marcin "beoArcha" Stefan. MIT License.'
SVG_PATH = "{http://www.w3.org/2000/svg}path"
SVG_ROOT = "{http://www.w3.org/2000/svg}svg"
QUADRANTS = {"tl": (0, 0), "tr": (1200, 0), "bl": (0, 2380), "br": (1200, 2380)}
NAMES = ((1, FAMILY), (3, UNIQUE_ID), (4, "oXySol Mono Regular"), (6, "oXySolMono-Regular"), (16, FAMILY))
PREVIEW_SIZES = ((6, 12), (10, 24), None)


class BuildError(Exception):
    """A rule of the style guide or of the base font is broken; the message names the file or key."""

    def __init__(self, where, rule):
        super().__init__(f"{where}: {rule}")


def is_mosaic(key):
    return key.rsplit("/", 1)[-1].startswith("mosaic-")


def is_front(key):
    return key.startswith("fronts/")


def cell_size(key):
    return MOSAIC if is_mosaic(key) else CELL


def local_name(tag):
    return tag.rsplit("}", 1)[-1]


def read_root(path):
    try:
        return ET.parse(path).getroot()
    except ET.ParseError as error:
        raise BuildError(path, f"not well-formed XML: {error}") from error


def check_frame(path, root, key):
    if root.tag != SVG_ROOT:
        raise BuildError(path, f"the root element must be <svg> in the SVG namespace, found <{local_name(root.tag)}>")
    for name in root.attrib:
        if name != "viewBox":
            raise BuildError(path, f"attribute `{name}` is not allowed on <svg>; only viewBox (no width or height)")
    width, height = cell_size(key)
    wanted = f"0 0 {width} {height}"
    found = " ".join(root.get("viewBox", "").split())
    if found != wanted:
        raise BuildError(path, f"viewBox must be `{wanted}`, found `{found}`")


def check_path_element(path, element):
    if element.tag != SVG_PATH:
        raise BuildError(path, f"only <path> elements are allowed, found <{local_name(element.tag)}>")
    if len(element):
        raise BuildError(path, "a <path> may not contain elements")
    for name in element.attrib:
        if name not in ("d", "fill"):
            raise BuildError(path, f"attribute `{name}` is not allowed; use only `d` and `fill` (fill only, no strokes)")
    if element.get("fill") not in (None, "#000"):
        raise BuildError(path, f"fill must be omitted or #000, found `{element.get('fill')}`")
    if not element.get("d"):
        raise BuildError(path, "a <path> has no `d` data")


def outline_of(shape):
    pen = BoundsPen(None)
    shape.draw(pen)
    return pen.bounds


def check_inside(path, key, bounds):
    if bounds is None:
        raise BuildError(path, "the glyph has no ink")
    if is_front(key) and not is_mosaic(key):
        x0, y0, x1, y1 = SAFE_AREA
        area = "safe area"
    else:
        x0, y0 = 0, 0
        x1, y1 = cell_size(key)
        area = "box"
    xmin, ymin, xmax, ymax = bounds
    if xmin < x0 or ymin < y0 or xmax > x1 or ymax > y1:
        raise BuildError(
            path,
            f"ink outside the {area} x {x0}…{x1}, y {y0}…{y1}; found x {xmin:.0f}…{xmax:.0f}, y {ymin:.0f}…{ymax:.0f}",
        )


def read_shape(path, key):
    root = read_root(path)
    check_frame(path, root, key)
    shape = pathops.Path()
    pen = shape.getPen()
    for element in root:
        check_path_element(path, element)
        try:
            parse_path(element.get("d"), pen)
        except Exception as error:
            raise BuildError(path, f"cannot read the path data: {error}") from error
    shape = pathops.simplify(shape, fix_winding=True, keep_starting_points=False)
    check_inside(path, key, outline_of(shape))
    return shape


def rectangle(x0, y0, x1, y1):
    shape = pathops.Path()
    pen = shape.getPen()
    pen.moveTo((x0, y0))
    pen.lineTo((x1, y0))
    pen.lineTo((x1, y1))
    pen.lineTo((x0, y1))
    pen.closePath()
    return shape


def cut_quadrant(shape, corner):
    x0, y0 = QUADRANTS[corner]
    window = rectangle(x0, y0, x0 + CELL[0], y0 + CELL[1])
    piece = pathops.op(shape, window, pathops.PathOp.INTERSECTION, fix_winding=True, keep_starting_points=False)
    return piece, (x0, y0)


def outline_glyph(shape, origin):
    x0, y0 = origin
    pen = TTGlyphPen(None)
    to_font = TransformPen(Cu2QuPen(pen, max_err=1.0), (1, 0, 0, -1, -x0, ASCENT + y0))
    shape.draw(to_font)
    return pen.glyph()


def check_base(font, path):
    head, hhea, os2 = font["head"], font["hhea"], font["OS/2"]
    checks = (
        ("units per em", head.unitsPerEm, UPM),
        ("ascent", hhea.ascent, ASCENT),
        ("descent", -hhea.descent, DESCENT),
        ("line gap", hhea.lineGap, 0),
        ("typo ascent", os2.sTypoAscender, ASCENT),
        ("typo descent", -os2.sTypoDescender, DESCENT),
        ("advance of A", font["hmtx"]["A"][0], ADVANCE),
    )
    for label, found, wanted in checks:
        if found != wanted:
            raise BuildError(path, f"{label} is {found}, the card glyph box needs {wanted} (Cascadia Mono cell, X-01)")
    if not font["post"].isFixedPitch:
        raise BuildError(path, "the base font is not monospaced")


def add_glyphs(font, glyphs):
    order = font.getGlyphOrder()
    existing = set(order)
    for name, _, _ in glyphs:
        if name in existing:
            raise BuildError(name, "a glyph of this name already exists in the base font")
    font.setGlyphOrder(order + [name for name, _, _ in glyphs])
    glyf, hmtx = font["glyf"], font["hmtx"]
    for name, code_point, glyph in glyphs:
        glyph.recalcBounds(glyf)
        glyf.glyphs[name] = glyph
        hmtx.metrics[name] = (ADVANCE, glyph.xMin if glyph.numberOfContours else 0)
        for table in font["cmap"].tables:
            if table.isUnicode():
                table.cmap[code_point] = name


def rename_family(font):
    names = font["name"]
    names.removeNames(nameID=7)
    for record in [record for record in names.names if record.nameID == 0]:
        names.setName(
            record.toUnicode() + COPYRIGHT_LINE, 0, record.platformID, record.platEncID, record.langID
        )
    for name_id, text in NAMES:
        names.removeNames(nameID=name_id)
        names.setName(text, name_id, 3, 1, 0x409)
        names.setName(text, name_id, 1, 0, 0)


def cell_html(code_point, width, height, font_px):
    return (
        f'<span class="cell" style="font-size:{font_px:.2f}px;width:{width:.2f}px;'
        f'height:{height:.2f}px;line-height:{height:.2f}px">&#x{code_point:04X};</span>'
    )


def sample_html(code_point, size):
    if size is None:
        font_px = 96.0
        width = font_px * ADVANCE / UPM
        height = font_px * CELL[1] / UPM
        return cell_html(code_point, width, height, font_px)
    width, height = size
    return cell_html(code_point, width, height, width * UPM / ADVANCE)


def mosaic_html(pieces, size):
    cells = []
    for code_point in pieces:
        cells.append(sample_html(code_point, size) if code_point is not None else "")
    columns = "" if size is None else f' style="grid-template-columns:repeat(2,{size[0]}px)"'
    return f'<div class="grid"{columns}>' + "".join(cells) + "</div>"


def preview_rows(glyphs):
    rows = []
    mosaics = {}
    for key, code_point in sorted(glyphs):
        if is_mosaic(key):
            mosaics.setdefault(key.rsplit("-", 1)[0], {})[key.rsplit("-", 1)[1]] = code_point
            continue
        samples = "".join(f"<td>{sample_html(code_point, size)}</td>" for size in PREVIEW_SIZES)
        rows.append(f"<tr><th>{html.escape(key)}</th><td>U+{code_point:04X}</td>{samples}</tr>")
    for stem, corners in sorted(mosaics.items()):
        pieces = [corners.get(corner) for corner in ("tl", "tr", "bl", "br")]
        codes = ", ".join(f"U+{code:04X}" for code in pieces if code is not None)
        samples = "".join(f"<td>{mosaic_html(pieces, size)}</td>" for size in PREVIEW_SIZES)
        rows.append(f"<tr><th>{html.escape(stem)}-(tl, tr, bl, br)</th><td>{codes}</td>{samples}</tr>")
    return rows


def write_preview(path, font_bytes, glyphs):
    encoded = base64.b64encode(font_bytes).decode("ascii")
    groups = {}
    for key, code_point in glyphs:
        groups.setdefault(key.rsplit("/", 1)[0], []).append((key, code_point))
    sections = []
    for group in sorted(groups, key=lambda name: (not name.startswith("fronts"), name)):
        rows = "".join(preview_rows(groups[group]))
        sections.append(
            f"<section><h2>{html.escape(group)}</h2><table><thead><tr><th>key</th><th>code point</th>"
            "<th>6 × 12 px</th><th>10 × 24 px</th><th>large</th></tr></thead>"
            f"<tbody>{rows}</tbody></table></section>"
        )
    body = "".join(sections) or "<p>No card glyphs are drawn yet; only the base font is built.</p>"
    page = PREVIEW_TEMPLATE.format(encoded=encoded, body=body)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(page, encoding="utf-8")


PREVIEW_TEMPLATE = """<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>oXySol Mono card glyphs</title>
<style>
@font-face {{ font-family: "oXySol Mono"; src: url(data:font/ttf;base64,{encoded}) format("truetype"); }}
:root {{ color-scheme: light; }}
body {{ background: #fff; color: #000; font: 14px system-ui, sans-serif; margin: 16px; }}
table {{ border-collapse: collapse; margin-bottom: 24px; }}
th, td {{ border: 1px solid #ccc; padding: 4px 8px; text-align: left; vertical-align: middle; }}
.cell {{ display: inline-block; overflow: hidden; white-space: nowrap; background: #fff; color: #000;
  font-family: "oXySol Mono", monospace; vertical-align: top; }}
.grid {{ display: inline-grid; grid-template-columns: repeat(2, auto); }}
</style></head>
<body>
<h1>oXySol Mono card glyphs</h1>
<p>Solid black on white. Each sample is one cell at 6 × 12 px, 10 × 24 px, or large (96 px font size).
Mosaics are shown as a 2 × 2 block.</p>
{body}
</body></html>
"""


def parse_glyph_arguments(entries):
    parsed = []
    for key, code_text, svg in entries:
        try:
            code_point = int(code_text, 16)
        except ValueError as error:
            raise BuildError(key, f"code point `{code_text}` is not hexadecimal") from error
        if not CODE_MIN <= code_point <= CODE_MAX:
            raise BuildError(key, f"code point U+{code_point:04X} lies outside U+E6BC–U+E6FF")
        parsed.append((key, code_point, Path(svg)))
    return parsed


def build(args):
    base_path = Path(args.base)
    font = TTFont(base_path, recalcTimestamp=False)
    check_base(font, base_path)
    built = []
    for key, code_point, svg in parse_glyph_arguments(args.glyph):
        shape = read_shape(svg, key)
        origin = (0, 0)
        if is_mosaic(key):
            shape, origin = cut_quadrant(shape, key.rsplit("-", 1)[1])
        built.append((f"uni{code_point:04X}", code_point, outline_glyph(shape, origin), key))
    add_glyphs(font, [(name, code_point, glyph) for name, code_point, glyph, _ in built])
    rename_family(font)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    font.save(out)
    write_preview(
        Path(args.preview),
        out.read_bytes(),
        [(key, code_point) for _, code_point, _, key in built],
    )
    print(f"built {out} with {len(built)} card glyph(s)")


def parse_args(argv):
    parser = argparse.ArgumentParser(description="Build the oXySol Mono card font.")
    parser.add_argument("--base", required=True, help="the Cascadia Mono base font")
    parser.add_argument("--out", required=True, help="the font to write")
    parser.add_argument("--preview", required=True, help="the contact sheet to write")
    parser.add_argument(
        "--glyph",
        nargs=3,
        action="append",
        default=[],
        metavar=("KEY", "CODE_POINT", "SVG"),
        help="one registry key with its hexadecimal code point and SVG file",
    )
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv)
    try:
        build(args)
    except BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
