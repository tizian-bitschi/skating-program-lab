"""Regression check: a wrong source edition must fail before table extraction."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from verify_sov_source import source_fixture


class SourceEditionTest(unittest.TestCase):
    def test_wrong_pdf_never_reaches_extractor_or_fallback(self):
        with tempfile.TemporaryDirectory() as directory:
            pdf = Path(directory) / "wrong-edition.pdf"
            pdf.write_bytes(b"%PDF-1.7 a different source edition")
            with patch("verify_sov_source.subprocess.run") as extractor:
                with self.assertRaisesRegex(ValueError, "Wrong PDF edition/hash"):
                    source_fixture(pdf)
                extractor.assert_not_called()


if __name__ == "__main__":
    unittest.main()
