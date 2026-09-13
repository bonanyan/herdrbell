import AppKit
import SwiftUI

/// Rasterizes a `StatusIcon` tinted with the scheme's color into a PNG file,
/// for surfaces outside SwiftUI (macOS notification attachments).
enum StatusIconImage {
    static func pngFileURL(for icon: StatusIcon, tint: Color, pixelSize: CGFloat = 256) -> URL? {
        guard
            let rep = render(icon: icon, tint: NSColor(tint), pixelSize: pixelSize),
            let png = rep.representation(using: .png, properties: [:]),
            let cache = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?
                .appendingPathComponent("dev.herdrbell.HerdrBell/NotificationIcons", isDirectory: true)
        else { return nil }
        try? FileManager.default.createDirectory(at: cache, withIntermediateDirectories: true)
        let url = cache.appendingPathComponent("\(key(for: icon, tint: tint, pixelSize: pixelSize)).png")
        try? png.write(to: url)
        return url
    }

    private static func key(for icon: StatusIcon, tint: Color, pixelSize: CGFloat) -> String {
        let base: String
        switch icon {
        case .systemSymbol(let name):
            base = "symbol-\(name)"
        case .asset(let name, _):
            base = "asset-\(name)"
        }
        let color = NSColor(tint).usingColorSpace(.sRGB) ?? NSColor(tint)
        let hex = String(
            format: "%02X%02X%02X",
            Int((color.redComponent * 255).rounded()),
            Int((color.greenComponent * 255).rounded()),
            Int((color.blueComponent * 255).rounded())
        )
        return "\(base)-\(hex)-\(Int(pixelSize))"
    }

    private static func render(icon: StatusIcon, tint: NSColor, pixelSize: CGFloat) -> NSBitmapImageRep? {
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
        )
        guard let rep else { return nil }
        rep.size = NSSize(width: pixelSize, height: pixelSize)
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
        let rect = CGRect(origin: .zero, size: NSSize(width: pixelSize, height: pixelSize))
        switch icon {
        case .systemSymbol(let name):
            drawSymbol(name: name, in: rect)
        case .asset(let name, let fallback):
            if let asset = NSImage(named: NSImage.Name(name)) {
                NSGraphicsContext.current?.imageInterpolation = .high
                asset.draw(in: rect, from: .zero, operation: .sourceOver, fraction: 1)
            } else {
                drawSymbol(name: fallback, in: rect)
            }
        }
        tint.setFill()
        rect.fill(using: .sourceAtop)
        NSGraphicsContext.restoreGraphicsState()
        return rep
    }

    private static func drawSymbol(name: String, in rect: CGRect) {
        guard
            let symbol = NSImage(systemSymbolName: name, accessibilityDescription: nil),
            let configured = symbol.withSymbolConfiguration(
                NSImage.SymbolConfiguration(pointSize: rect.height * 0.72, weight: .semibold)
            )
        else { return }
        let size = configured.size
        configured.draw(
            in: NSRect(
                x: rect.midX - size.width / 2,
                y: rect.midY - size.height / 2,
                width: size.width,
                height: size.height
            ),
            from: .zero,
            operation: .sourceOver,
            fraction: 1
        )
    }
}
