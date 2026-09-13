import AppKit
import Foundation

func makeBitmapRep(pixelSize: CGFloat) -> NSBitmapImageRep {
    let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: Int(pixelSize),
        pixelsHigh: Int(pixelSize),
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: 0,
        bitsPerPixel: 0
    )!
    rep.size = NSSize(width: pixelSize, height: pixelSize)
    return rep
}

func renderIcon(pixelSize: CGFloat) -> NSBitmapImageRep {
    let size = NSSize(width: pixelSize, height: pixelSize)
    let rep = makeBitmapRep(pixelSize: pixelSize)
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)

    let rect = CGRect(origin: .zero, size: size)
    let radius = pixelSize * 0.225
    let shape = NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius)
    shape.addClip()

    let gradient = NSGradient(
        starting: NSColor(red: 0.16, green: 0.15, blue: 0.42, alpha: 1),
        ending: NSColor(red: 0.44, green: 0.24, blue: 0.74, alpha: 1)
    )
    gradient?.draw(in: shape, angle: 270)

    if let base = NSImage(systemSymbolName: "bell.fill", accessibilityDescription: nil) {
        let config = NSImage.SymbolConfiguration(pointSize: pixelSize * 0.52, weight: .semibold)
            .applying(NSImage.SymbolConfiguration(paletteColors: [.white]))
        if let symbol = base.withSymbolConfiguration(config) {
            let symbolSize = symbol.size
            let origin = NSPoint(
                x: (pixelSize - symbolSize.width) / 2,
                y: (pixelSize - symbolSize.height) / 2 - pixelSize * 0.02
            )
            symbol.draw(
                in: NSRect(origin: origin, size: symbolSize),
                from: .zero,
                operation: .sourceOver,
                fraction: 1
            )
            let dotRadius = pixelSize * 0.085
            let dotCenter = NSPoint(
                x: origin.x + symbolSize.width * 0.78,
                y: origin.y + symbolSize.height * 0.82
            )
            NSColor(red: 1.0, green: 0.35, blue: 0.35, alpha: 1).setFill()
            let dot = NSBezierPath(ovalIn: NSRect(
                x: dotCenter.x - dotRadius,
                y: dotCenter.y - dotRadius,
                width: dotRadius * 2,
                height: dotRadius * 2
            ))
            dot.fill()
            NSColor.white.withAlphaComponent(0.9).setStroke()
            dot.lineWidth = pixelSize * 0.015
            dot.stroke()
        }
    }

    NSGraphicsContext.restoreGraphicsState()
    return rep
}

func renderFromMaster(_ master: NSImage, pixelSize: CGFloat) -> NSBitmapImageRep {
    let size = NSSize(width: pixelSize, height: pixelSize)
    let rep = makeBitmapRep(pixelSize: pixelSize)
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    NSGraphicsContext.current?.imageInterpolation = .high
    master.draw(
        in: CGRect(origin: .zero, size: size),
        from: .zero,
        operation: .sourceOver,
        fraction: 1
    )
    NSGraphicsContext.restoreGraphicsState()
    return rep
}

func savePNG(_ rep: NSBitmapImageRep, to url: URL) {
    guard let png = rep.representation(using: .png, properties: [:]) else {
        fatalError("failed to encode PNG for \(url.lastPathComponent)")
    }
    try? png.write(to: url)
}

var masterPath: String?
var outputDir = "Sources/Resources/Assets.xcassets/AppIcon.appiconset"
var args = Array(CommandLine.arguments.dropFirst())
var index = 0
while index < args.count {
    switch args[index] {
    case "--master":
        index += 1
        if index < args.count { masterPath = args[index] }
    case "--out":
        index += 1
        if index < args.count { outputDir = args[index] }
    default:
        outputDir = args[index]
    }
    index += 1
}

let outURL = URL(fileURLWithPath: outputDir)
try? FileManager.default.createDirectory(at: outURL, withIntermediateDirectories: true)

var master: NSImage?
if let masterPath {
    guard let loaded = NSImage(contentsOfFile: masterPath) else {
        fatalError("cannot load master image at \(masterPath)")
    }
    master = loaded
    print("slicing master image: \(masterPath)")
}

let sizes: [CGFloat] = [16, 32, 64, 128, 256, 512, 1024]
for size in sizes {
    let image = master.map { renderFromMaster($0, pixelSize: size) }
        ?? renderIcon(pixelSize: size)
    let url = outURL.appendingPathComponent("icon_\(Int(size)).png")
    savePNG(image, to: url)
    print("wrote \(url.path)")
}

let contents = """
{
  "images" : [
    { "filename" : "icon_16.png", "idiom" : "mac", "scale" : "1x", "size" : "16x16" },
    { "filename" : "icon_32.png", "idiom" : "mac", "scale" : "2x", "size" : "16x16" },
    { "filename" : "icon_32.png", "idiom" : "mac", "scale" : "1x", "size" : "32x32" },
    { "filename" : "icon_64.png", "idiom" : "mac", "scale" : "2x", "size" : "32x32" },
    { "filename" : "icon_128.png", "idiom" : "mac", "scale" : "1x", "size" : "128x128" },
    { "filename" : "icon_256.png", "idiom" : "mac", "scale" : "2x", "size" : "128x128" },
    { "filename" : "icon_256.png", "idiom" : "mac", "scale" : "1x", "size" : "256x256" },
    { "filename" : "icon_512.png", "idiom" : "mac", "scale" : "2x", "size" : "256x256" },
    { "filename" : "icon_512.png", "idiom" : "mac", "scale" : "1x", "size" : "512x512" },
    { "filename" : "icon_1024.png", "idiom" : "mac", "scale" : "2x", "size" : "512x512" }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
"""
try contents.write(to: outURL.appendingPathComponent("Contents.json"), atomically: true, encoding: .utf8)
print("wrote Contents.json")
