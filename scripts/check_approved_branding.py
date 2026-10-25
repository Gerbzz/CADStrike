"""Approved 2026-10-25 branding contract. Requires Pillow; does not modify files."""
from pathlib import Path
from hashlib import sha256
import json
import unittest
from PIL import Image
ROOT = Path(__file__).resolve().parents[1]
APP = 'CADStrike'
APPROVED = 'be30c3ebefc7fadb8300a06a4a3e589b6e9a56f123b0d8313a59c5ac206e80f9'
class ApprovedBranding(unittest.TestCase):
    def test_exact_approved_source_and_readme(self):
        p = ROOT / 'assets/branding' / (APP + '.png')
        self.assertEqual(sha256(p.read_bytes()).hexdigest(), APPROVED)
        with Image.open(p) as image:
            self.assertEqual(image.size, (512,512))
        self.assertIn('assets/branding/' + APP + '.png', (ROOT/'README.md').read_text())
        self.assertEqual(json.loads((p.parent/'provenance.json').read_text())['sha256'], APPROVED)
    def test_runtime_derivative_and_binding(self):
        source = Image.open(ROOT/'assets/branding'/(APP+'.png')).convert('RGBA')
        raw = ROOT/'assets/branding'/(APP+'-128.rgba')
        self.assertTrue(raw.exists(), 'runtime derivative absent')
        self.assertEqual(raw.read_bytes(), source.resize((128,128),Image.Resampling.LANCZOS).tobytes())
        self.assertIn(APP+'-128.rgba', (ROOT/'src/bin/cadstrike-ui.rs').read_text())

if __name__ == "__main__":
    unittest.main()
