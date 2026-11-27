"""scripts/manifest.py 的測試：python -m unittest discover -s scripts"""
import datetime
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import manifest

NOW = datetime.datetime(2026, 11, 27, 8, 0, tzinfo=datetime.timezone.utc)


class ManifestTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dist = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def add(self, name: str, data: bytes = b"data", sig: str | None = "U0lH\n"):
        (self.dist / name).write_bytes(data)
        if sig is not None:
            (self.dist / f"{name}.sig").write_text(sig, encoding="utf-8")

    def test_basic(self):
        self.add("z-overlay.exe", b"exe")
        self.add("z-overlay.exe.sha256", b"x", sig=None)
        m = manifest.build("zoosewu/z-overlay", "v0.2.0", self.dist, {"windows-x86_64": "z-overlay.exe"}, "### 修正\n- a\n", NOW)
        self.assertEqual(m["version"], "0.2.0")
        self.assertEqual(m["notes"], "### 修正\n- a")
        self.assertEqual(m["pub_date"], "2026-11-27T08:00:00Z")
        p = m["platforms"]["windows-x86_64"]
        self.assertEqual(p["url"], "https://github.com/zoosewu/z-overlay/releases/download/v0.2.0/z-overlay.exe")
        self.assertEqual(p["signature"], "U0lH")
        self.assertEqual(p["sha256"], hashlib.sha256(b"exe").hexdigest())

    def test_glob(self):
        self.add("zoo-downloader_0.4.0_x64-setup.exe")
        self.add("zoo-downloader_0.4.0_x64_en-US.msi")
        m = manifest.build("zoosewu/zoo-downloader", "v0.4.0", self.dist, {"windows-x86_64": "*_x64-setup.exe"}, "", NOW)
        self.assertTrue(m["platforms"]["windows-x86_64"]["url"].endswith("zoo-downloader_0.4.0_x64-setup.exe"))

    def test_prerelease_tag(self):
        m = manifest.build("zoosewu/x", "v0.7.0-rc.1", self.dist, {}, "", NOW)
        self.assertEqual(m["version"], "0.7.0-rc.1")

    def test_docker_has_empty_platforms(self):
        m = manifest.build("zoosewu/zkaraver", "v1.2.3", self.dist, {}, "", NOW)
        self.assertEqual(m["platforms"], {})

    def test_errors(self):
        self.add("a.exe")
        self.add("b.exe")
        self.add("nosig.exe", sig=None)
        cases = [
            ("0.2.0", {}),                                   # 沒有 v
            ("v0.2", {}),                                    # 版本不完整
            ("v0.2.0", {"win-x64": "a.exe"}),                # 平台鍵錯誤
            ("v0.2.0", {"windows-x86_64": "*.exe"}),         # 對到多個
            ("v0.2.0", {"windows-x86_64": "missing.exe"}),   # 沒有對到
            ("v0.2.0", {"windows-x86_64": "nosig.exe"}),     # 沒有簽章
        ]
        for tag, platforms in cases:
            with self.subTest(tag=tag, platforms=platforms):
                with self.assertRaises(manifest.ManifestError):
                    manifest.build("zoosewu/x", tag, self.dist, platforms, "", NOW)

    def test_print_assets_ignores_signatures(self):
        self.add("app.zip", sig=None)
        import contextlib, io
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            code = manifest.main(["--repo", "zoosewu/x", "--tag", "v1.0.0", "--dist", str(self.dist),
                                  "--platforms", json.dumps({"windows-x86_64": "*.zip"}), "--print-assets"])
        self.assertEqual(code, 0)
        self.assertEqual(Path(buf.getvalue().strip()).name, "app.zip")

    def test_main_writes_file(self):
        self.add("app.zip")
        out = self.dist / "latest.json"
        code = manifest.main(["--repo", "zoosewu/x", "--tag", "v1.0.0", "--dist", str(self.dist),
                              "--platforms", json.dumps({"windows-x86_64": "app.zip"}), "--out", str(out)])
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out.read_text(encoding="utf-8"))["version"], "1.0.0")


if __name__ == "__main__":
    unittest.main()
