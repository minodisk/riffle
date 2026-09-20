// Renders the SF Symbols used by Riffle's own macOS menu items into PNGs under
// `crates/app/icons/menu/`. Run by a developer only, never at build or run
// time:
//
//     swift tools/macos/export-menu-icons.swift
//
// Last run on macOS 26.6.2 (25G83). AppKit's symbol rasterisation can change
// between OS releases, so the committed PNGs are the source of truth; rerun
// this only when a symbol, its size, weight or colour changes.
//
// The symbols are drawn in a fixed neutral grey because muda never marks a
// custom menu image as a template and Tauri exposes no way to do so, so the
// icons cannot tint with the menu appearance. The grey stays legible in both
// light and dark mode.

import AppKit

let pointSize: CGFloat = 18
let scale: CGFloat = 2
let color = NSColor(srgbRed: 0x8E / 255, green: 0x8E / 255, blue: 0x93 / 255, alpha: 1)
let symbols = ["gearshape", "arrow.uturn.backward", "folder"]

let root = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
let directory = root.appendingPathComponent("crates/app/icons/menu")

for name in symbols {
    guard let symbol = NSImage(systemSymbolName: name, accessibilityDescription: nil) else {
        FileHandle.standardError.write("unknown SF Symbol: \(name)\n".data(using: .utf8)!)
        exit(1)
    }
    let configured = symbol.withSymbolConfiguration(
        NSImage.SymbolConfiguration(pointSize: pointSize, weight: .medium)
    )!
    let pixels = Int(pointSize * scale)
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: pixels,
        pixelsHigh: pixels,
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: 0,
        bitsPerPixel: 0
    ) else {
        exit(1)
    }
    rep.size = NSSize(width: pointSize, height: pointSize)
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    let size = configured.size
    let rect = NSRect(
        x: (pointSize - size.width) / 2,
        y: (pointSize - size.height) / 2,
        width: size.width,
        height: size.height
    )
    configured.draw(in: rect)
    color.set()
    rect.fill(using: .sourceAtop)
    NSGraphicsContext.restoreGraphicsState()
    guard let png = rep.representation(using: .png, properties: [:]) else { exit(1) }
    try png.write(to: directory.appendingPathComponent("\(name).png"))
}
