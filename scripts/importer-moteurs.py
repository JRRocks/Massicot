#!/usr/bin/env python3
"""Copies the PdfCraft engines Massicot reuses, and rewrites their manifests to stand alone.

Usage: importer-moteurs.py <pdfcraft checkout> <craft-libs checkout> <Massicot root>

- PdfCraft crates  -> crates/<name>/      (sources unchanged; tests, benches, examples, fuzz left out)
- PdfCraft vendor/ -> vendor/<name>/      (patched third-party crates, copied whole)
- craft-libs       -> vendor/craft-libs/<name>/ (the git dependencies PdfCraft pulls from craft-libs)

Copied crates are not workspace members: their manifests lose workspace inheritance (values are
copied from the source workspace), their dev-dependencies and their test/bench/example targets.
The checkouts must be at the revisions listed in ATTRIBUTION.md.
"""

import shutil
import sys
from pathlib import Path

import tomllib
import tomlkit

PDFCRAFT_CRATES = ["cos", "content", "organize", "fonts", "geom", "render", "ocr"]
PDFCRAFT_VENDOR = [
    "hayro", "hayro-syntax", "hayro-interpret", "hayro-cmap", "hayro-postscript",
    "hayro-jbig2", "lopdf", "vello_common", "vello_cpu",
]
CRAFT_LIBS = {"craft-pdf": "crates/pdf", "craft-stream-codecs": "crates/stream-codecs"}
# Features of craft-pdf whose optional dependencies are other craft-libs crates Massicot does not
# copy (SVG writing). Nothing in Massicot enables them.
DROPPED_FEATURES = {"craft-pdf": ["write-svg"]}
DROPPED_DEPS = {"craft-pdf": ["craft-svg", "krilla-svg", "usvg"], "pdfcraft-ocr": ["craft-image-codecs"]}
ADDED_DEPS = {"pdfcraft-ocr": {"png": "0.18"}}

# Source patches, each marked `Massicot patch:` in the copied file. The OCR crate only used
# craft-image-codecs (35,000 lines and a dozen image formats) to hand Tesseract a PNG: the `png`
# crate, already in the dependency graph, encodes it.
OLD_ENCODE_PNG = """fn encode_png(image: &OcrImage) -> Result<Vec<u8>, OcrError> {
    let img = craft_image_codecs::Image::from_u8(image.width, image.height, craft_image_codecs::ChannelLayout::Rgba, image.rgba.clone())
        .map_err(|e| OcrError::Process(format!("encoding the page image failed: {e}")))?;
    craft_image_codecs::encode(&img, craft_image_codecs::Format::Png, &craft_image_codecs::EncodeOptions::default())
        .map_err(|e| OcrError::Process(format!("encoding the page image failed: {e}")))
}"""
NEW_ENCODE_PNG = """fn encode_png(image: &OcrImage) -> Result<Vec<u8>, OcrError> {
    // Massicot patch: encoded with the `png` crate instead of craft-image-codecs.
    let failed = |e: png::EncodingError| OcrError::Process(format!("encoding the page image failed: {e}"));
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(failed)?;
    writer.write_image_data(&image.rgba).map_err(failed)?;
    writer.finish().map_err(failed)?;
    Ok(out)
}"""
SOURCE_PATCHES = {"crates/ocr/src/tesseract.rs": [(OLD_ENCODE_PNG, NEW_ENCODE_PNG)]}
# Files the copied crates embed with include_bytes!, with their licences.
ASSETS = ["assets/fonts"]
SKIPPED_DIRS = {"tests", "benches", "examples", "fuzz", "read-only-contract", "target"}


def load(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def copy_tree(src, dst, skip_dirs):
    if dst.exists():
        shutil.rmtree(dst)

    def ignore(directory, names):
        if Path(directory) == src:
            return [n for n in names if n in skip_dirs or n == "Cargo.lock"]
        return []

    shutil.copytree(src, dst, ignore=ignore)


def rel(from_dir, to_dir):
    import os
    return Path(os.path.relpath(to_dir, from_dir)).as_posix()


def resolve_dep(name, spec, ws_deps, ws_root, new_dir, locations):
    """A dependency without workspace inheritance, pointing at the copies Massicot holds."""
    if not (isinstance(spec, dict) and spec.get("workspace")):
        out = dict(spec) if isinstance(spec, dict) else spec
    else:
        base = ws_deps[name]
        out = dict(base) if isinstance(base, dict) else {"version": base}
        for key, value in spec.items():
            if key == "workspace":
                continue
            if key == "features":
                out["features"] = sorted(set(out.get("features", [])) | set(value))
            else:
                out[key] = value
    # Local and git dependencies point at Massicot's copies; registry dependencies stay as they are
    # (the root [patch.crates-io] substitutes the patched copies under vendor/, as PdfCraft does).
    if isinstance(out, dict) and ("path" in out or "git" in out):
        package = out.get("package", name)
        if package not in locations:
            raise SystemExit(f"{name}: dépendance non copiée ({out.get('path') or out.get('git')})")
        for key in ("git", "rev", "branch", "tag"):
            out.pop(key, None)
        out["path"] = rel(new_dir, locations[package])
    return out


def rewrite_manifest(src_manifest, ws, ws_root, new_dir, locations, crate):
    doc = tomlkit.parse(src_manifest.read_text(encoding="utf-8"))
    ws_package = ws["workspace"].get("package", {})
    ws_deps = ws["workspace"].get("dependencies", {})
    package = doc["package"]
    for key in list(package.keys()):
        value = package[key]
        if isinstance(value, dict) and value.get("workspace"):
            package[key] = ws_package[key]
    package["publish"] = False
    package["autotests"] = False
    package["autobenches"] = False
    package["autoexamples"] = False
    for target in ("test", "bench", "example"):
        doc.pop(target, None)
    doc.pop("dev-dependencies", None)

    def fix_table(table):
        for name in list(table.keys()):
            if name in DROPPED_DEPS.get(crate, []):
                del table[name]
                continue
            spec = resolve_dep(name, table[name].unwrap() if hasattr(table[name], "unwrap") else table[name],
                               ws_deps, ws_root, new_dir, locations)
            if isinstance(spec, dict):
                inline = tomlkit.inline_table()
                inline.update(spec)
                spec = inline
            table[name] = spec

    for section in ("dependencies", "build-dependencies"):
        if section in doc:
            fix_table(doc[section])
    if "target" in doc:
        for cfg in list(doc["target"].keys()):
            t = doc["target"][cfg]
            t.pop("dev-dependencies", None)
            for section in ("dependencies", "build-dependencies"):
                if section in t:
                    fix_table(t[section])
            if len(t) == 0:
                del doc["target"][cfg]
        if len(doc["target"]) == 0:
            del doc["target"]
    if "features" in doc:
        for feature in DROPPED_FEATURES.get(crate, []):
            doc["features"].pop(feature, None)
    for name, spec in ADDED_DEPS.get(crate, {}).items():
        doc["dependencies"][name] = spec
    lints = ws["workspace"].get("lints")
    if "lints" in doc:
        del doc["lints"]
        if lints:
            # `unused_workspace_dependencies` means nothing outside a workspace (rustc warns about
            # an unknown lint): it is left out.
            doc["lints"] = {tool: {k: v for k, v in table.items() if k != "unused_workspace_dependencies"}
                            for tool, table in lints.items()}
    return tomlkit.dumps(doc)


def main():
    pdfcraft, craft_libs, root = (Path(a).resolve() for a in sys.argv[1:4])
    pdfcraft_ws = load(pdfcraft / "Cargo.toml")
    craft_ws = load(craft_libs / "Cargo.toml")

    locations = {}
    for c in PDFCRAFT_CRATES:
        name = load(pdfcraft / "crates" / c / "Cargo.toml")["package"]["name"]
        locations[name] = root / "crates" / c
    for v in PDFCRAFT_VENDOR:
        locations[v] = root / "vendor" / v
    for name, sub in CRAFT_LIBS.items():
        locations[name] = root / "vendor" / "craft-libs" / Path(sub).name

    for c in PDFCRAFT_CRATES:
        src, dst = pdfcraft / "crates" / c, root / "crates" / c
        copy_tree(src, dst, SKIPPED_DIRS)
        name = load(src / "Cargo.toml")["package"]["name"]
        (dst / "Cargo.toml").write_text(
            rewrite_manifest(src / "Cargo.toml", pdfcraft_ws, pdfcraft, dst, locations, name), encoding="utf-8")
    for v in PDFCRAFT_VENDOR:
        copy_tree(pdfcraft / "vendor" / v, root / "vendor" / v, set())
    for a in ASSETS:
        copy_tree(pdfcraft / a, root / a, set())
    for name, sub in CRAFT_LIBS.items():
        src, dst = craft_libs / sub, locations[name]
        copy_tree(src, dst, SKIPPED_DIRS)
        (dst / "Cargo.toml").write_text(
            rewrite_manifest(src / "Cargo.toml", craft_ws, craft_libs, dst, locations, name), encoding="utf-8")
    for file, patches in SOURCE_PATCHES.items():
        path = root / file
        text = path.read_text(encoding="utf-8")
        for old, new in patches:
            if text.count(old) != 1:
                raise SystemExit(f"{file} : le passage à corriger a changé")
            text = text.replace(old, new)
        path.write_text(text, encoding="utf-8")
    print("Copie terminée :", ", ".join(sorted(locations)))


if __name__ == "__main__":
    main()
