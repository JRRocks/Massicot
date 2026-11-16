#!/usr/bin/env python3
"""Exports Massicot's icon (assets/icone/massicot.svg) to the files the build and the installer use.

Run by hand after a change to the drawing (the CI does not run it; its outputs are committed):
    python3 scripts/icone.py
Needs Node.js with Playwright and its Chromium (to rasterise the SVG), and Pillow.

Outputs, in assets/icone/:
- massicot.ico: 16 to 256 px, embedded in massicot.exe and used by the installer;
- assistant-petit-*.bmp: the installer's small header image (Inno Setup sizes, 100 % to 250 %);
- assistant-grand-*.bmp: the installer's side image on the last page (same scales).
"""

import io
import json
import pathlib
import subprocess
import tempfile

from PIL import Image

RACINE = pathlib.Path(__file__).resolve().parent.parent
DOSSIER = RACINE / "assets" / "icone"
SVG = DOSSIER / "massicot.svg"

TAILLES_ICO = [16, 20, 24, 32, 40, 48, 64, 128, 256]
# Inno Setup's recommended bitmap sizes for 100, 150, 200 and 250 % scaling (it picks the closest).
PETITES = [(55, 55), (83, 80), (110, 106), (138, 140)]
GRANDES = [(164, 314), (246, 459), (328, 628), (410, 797)]

# Rasterises the SVG at each requested size with Chromium (transparent background).
RENDU = r"""
const { chromium } = require('playwright');
const fs = require('fs');
(async () => {
  const [svg, sortie, tailles] = process.argv.slice(2);
  const source = fs.readFileSync(svg, 'utf8');
  const b = await chromium.launch();
  const p = await b.newPage();
  for (const t of JSON.parse(tailles)) {
    await p.setViewportSize({ width: t, height: t });
    await p.setContent(`<html><body style="margin:0;background:transparent">
      <img src="data:image/svg+xml;base64,${Buffer.from(source).toString('base64')}" width="${t}" height="${t}"></body></html>`);
    await p.screenshot({ path: `${sortie}/${t}.png`, omitBackground: true });
  }
  await b.close();
})();
"""


def rasteriser(tailles):
    """PNG images of the icon, by size."""
    with tempfile.TemporaryDirectory() as tmp:
        script = pathlib.Path(tmp) / "rendu.js"
        script.write_text(RENDU)
        subprocess.run(["node", str(script), str(SVG), tmp, json.dumps(sorted(set(tailles)))], check=True)
        return {t: Image.open(io.BytesIO((pathlib.Path(tmp) / f"{t}.png").read_bytes())).convert("RGBA") for t in set(tailles)}


def degrade(largeur, hauteur):
    """The icon's blue, as a diagonal gradient (side image background)."""
    haut, bas = (0x2F, 0x6F, 0xD6), (0x1B, 0x3F, 0x86)
    image = Image.new("RGB", (largeur, hauteur))
    pixels = image.load()
    for y in range(hauteur):
        for x in range(largeur):
            t = (x / largeur * 0.35 + y / hauteur * 0.65)
            pixels[x, y] = tuple(round(h + (b - h) * t) for h, b in zip(haut, bas))
    return image


def main():
    cotes_petites = [round(min(w, h) * 0.9) for w, h in PETITES]
    cotes_grandes = [round(w * 0.6) for w, _ in GRANDES]
    images = rasteriser(TAILLES_ICO + cotes_petites + cotes_grandes)

    ico = [images[t] for t in TAILLES_ICO]
    ico[-1].save(DOSSIER / "massicot.ico", format="ICO", sizes=[(t, t) for t in TAILLES_ICO], append_images=ico[:-1])

    for (w, h), cote in zip(PETITES, cotes_petites):
        fond = Image.new("RGB", (w, h), "white")
        fond.paste(images[cote], ((w - cote) // 2, (h - cote) // 2), images[cote])
        fond.save(DOSSIER / f"assistant-petit-{w}x{h}.bmp")

    for (w, h), cote in zip(GRANDES, cotes_grandes):
        fond = degrade(w, h)
        fond.paste(images[cote], ((w - cote) // 2, round(h * 0.55) - cote // 2), images[cote])
        fond.save(DOSSIER / f"assistant-grand-{w}x{h}.bmp")

    print("Icône exportée dans", DOSSIER.relative_to(RACINE))


if __name__ == "__main__":
    main()
