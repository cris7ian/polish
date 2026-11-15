import AppKit
import Foundation

let destination = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
try FileManager.default.createDirectory(at: destination, withIntermediateDirectories: true)
let symbols = [
    "undo": "arrow.uturn.backward",
    "redo": "arrow.uturn.forward",
    "copy": "document.on.document",
    "rework": "arrow.triangle.2.circlepath",
    "polish": "sparkles",
    "shorten": "text.alignleft",
    "ideas": "ellipsis",
]
for (name, symbolName) in symbols {
    guard let symbol = NSImage(systemSymbolName: symbolName, accessibilityDescription: nil) else {
        fatalError("SF Symbol not available: \(symbolName)")
    }
    let ink: NSColor = name == "polish" ? .white : NSColor(srgbRed: 0.47, green: 0.36, blue: 0.35, alpha: 1)
    let config = NSImage.SymbolConfiguration(pointSize: 54, weight: .medium)
        .applying(NSImage.SymbolConfiguration(paletteColors: [ink]))
    guard let image = symbol.withSymbolConfiguration(config),
          let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 72, pixelsHigh: 72,
              bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
              colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0),
          let context = NSGraphicsContext(bitmapImageRep: bitmap) else {
        fatalError("Could not render SF Symbol: \(symbolName)")
    }
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = context
    context.imageInterpolation = .high
    let size = image.size
    image.draw(in: NSRect(x: (72 - size.width) / 2, y: (72 - size.height) / 2,
                          width: size.width, height: size.height))
    NSGraphicsContext.restoreGraphicsState()
    guard let png = bitmap.representation(using: .png, properties: [:]) else { fatalError("PNG encoding failed") }
    try png.write(to: destination.appendingPathComponent("\(name).png"))
}
