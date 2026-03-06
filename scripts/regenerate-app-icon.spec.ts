import { describe, expect, it } from "bun:test";
import {
  buildIconSvg,
  type GlyphOutline,
  validationRasterSizes,
} from "./regenerate-app-icon";

describe("regenerate app icon", () => {
  it("builds a path-only svg without text nodes", () => {
    const glyph: GlyphOutline = {
      bounds: {
        height: 200,
        maxX: 100,
        maxY: 200,
        minX: 0,
        minY: 0,
        width: 100,
      },
      fontName: "HelveticaNeue-Bold",
      pathData: "M0 0 L100 0 L100 200 L0 200 Z",
    };

    const svg = buildIconSvg(glyph);

    expect(svg).toContain("<path");
    expect(svg).toContain('fill="#FFFFFF"');
    expect(svg).toContain(`data-font="${glyph.fontName}"`);
    expect(svg).not.toContain("<text");
    expect(svg).toContain('transform="translate(');
  });

  it("validates the raster output at tiny macOS icon sizes", () => {
    expect(validationRasterSizes).toEqual([16, 32, 64, 128, 256, 512, 1024]);
  });
});
