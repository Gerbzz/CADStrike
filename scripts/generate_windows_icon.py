"""Deterministic packaging-only sizes of the unchanged approved logo (Pillow 12.3.0).

Run from any directory. Does not modify the canonical PNG or its provenance.
"""
from io import BytesIO
from pathlib import Path
import struct

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SIZES = (16, 24, 32, 48, 64, 128, 256)


def generate():
    source = Image.open(ROOT / "assets/branding/CADStrike.png").convert("RGBA")
    payloads = []
    entries = []
    offset = 6 + 16 * len(SIZES)
    for size in SIZES:
        output = BytesIO()
        source.resize((size, size), Image.Resampling.LANCZOS).save(output, format="PNG", compress_level=9)
        payload = output.getvalue()
        entries.append(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(payload), offset))
        payloads.append(payload)
        offset += len(payload)
    return struct.pack("<HHH", 0, 1, len(SIZES)) + b"".join(entries + payloads)


if __name__ == "__main__":
    (ROOT / "assets/branding/CADStrike.ico").write_bytes(generate())
