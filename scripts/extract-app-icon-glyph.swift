import CoreText
import Foundation

struct GlyphBounds: Codable {
  let minX: Double
  let minY: Double
  let maxX: Double
  let maxY: Double
  let width: Double
  let height: Double
}

struct GlyphOutline: Codable {
  let fontName: String
  let pathData: String
  let bounds: GlyphBounds
}

func formatNumber(_ value: CGFloat) -> String {
  let rounded = (Double(value) * 1000).rounded() / 1000
  var text = String(format: "%.3f", rounded)

  while text.contains(".") && (text.hasSuffix("0") || text.hasSuffix(".")) {
    text.removeLast()
  }

  return text == "-0" ? "0" : text
}

func svgPathData(from path: CGPath) -> String {
  var commands: [String] = []

  path.applyWithBlock { elementPointer in
    let element = elementPointer.pointee
    let points = element.points

    switch element.type {
    case .moveToPoint:
      commands.append(
        "M\(formatNumber(points[0].x)) \(formatNumber(points[0].y))"
      )
    case .addLineToPoint:
      commands.append(
        "L\(formatNumber(points[0].x)) \(formatNumber(points[0].y))"
      )
    case .addQuadCurveToPoint:
      commands.append(
        "Q\(formatNumber(points[0].x)) \(formatNumber(points[0].y)) \(formatNumber(points[1].x)) \(formatNumber(points[1].y))"
      )
    case .addCurveToPoint:
      commands.append(
        "C\(formatNumber(points[0].x)) \(formatNumber(points[0].y)) \(formatNumber(points[1].x)) \(formatNumber(points[1].y)) \(formatNumber(points[2].x)) \(formatNumber(points[2].y))"
      )
    case .closeSubpath:
      commands.append("Z")
    @unknown default:
      break
    }
  }

  return commands.joined(separator: " ")
}

func fail(_ message: String) -> Never {
  FileHandle.standardError.write(Data("\(message)\n".utf8))
  exit(1)
}

let args = CommandLine.arguments
let fontName = args.count > 1 ? args[1] : "HelveticaNeue-Bold"
let glyphCharacter = args.count > 2 ? args[2] : "R"

guard glyphCharacter.count == 1, let scalar = glyphCharacter.unicodeScalars.first else {
  fail("Expected a single glyph character.")
}

let font = CTFontCreateWithName(fontName as CFString, 1000, nil)
let resolvedFontName = CTFontCopyPostScriptName(font) as String

var character = UniChar(scalar.value)
var glyph = CGGlyph()
guard CTFontGetGlyphsForCharacters(font, &character, &glyph, 1) else {
  fail("Unable to resolve glyph '\(glyphCharacter)' for font '\(fontName)'.")
}

guard let path = CTFontCreatePathForGlyph(font, glyph, nil) else {
  fail("Unable to extract path for glyph '\(glyphCharacter)'.")
}

let box = path.boundingBoxOfPath
let outline = GlyphOutline(
  fontName: resolvedFontName,
  pathData: svgPathData(from: path),
  bounds: GlyphBounds(
    minX: Double(box.minX),
    minY: Double(box.minY),
    maxX: Double(box.maxX),
    maxY: Double(box.maxY),
    width: Double(box.width),
    height: Double(box.height)
  )
)

let encoder = JSONEncoder()
encoder.outputFormatting = [.sortedKeys]
let data = try encoder.encode(outline)
FileHandle.standardOutput.write(data)
