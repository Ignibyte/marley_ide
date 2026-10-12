//! The devices an agent can lay a page out as (#756), after the list in Chrome's `DevTools`.
//!
//! Each has its viewport in CSS pixels, its device pixels to the CSS pixel, and whether it is a
//! phone or tablet, which turns on the page's mobile layout rules. The user agent never changes.

/// A device a page can be laid out as.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Device {
    /// The name an agent asks for and the tab's chip shows.
    pub name: &'static str,
    /// The viewport's width in CSS pixels.
    pub width: u32,
    /// The viewport's height in CSS pixels.
    pub height: u32,
    /// Device pixels to the CSS pixel.
    pub scale: f64,
    /// A phone or tablet: the page's mobile layout rules apply.
    pub mobile: bool,
}

/// The devices, phones first.
pub const DEVICES: &[Device] = &[
    device("iPhone SE", 375, 667, 2.0, true),
    device("iPhone 15 Pro", 393, 852, 3.0, true),
    device("iPhone 15 Pro Max", 430, 932, 3.0, true),
    device("Pixel 8", 412, 915, 2.625, true),
    device("Galaxy S24", 360, 780, 3.0, true),
    device("iPad Mini", 768, 1024, 2.0, true),
    device("iPad Air", 820, 1180, 2.0, true),
    device("iPad Pro 13", 1024, 1366, 2.0, true),
    device("Surface Pro 7", 912, 1368, 2.0, true),
    device("Laptop", 1366, 768, 1.0, false),
    device("Desktop", 1920, 1080, 1.0, false),
];

const fn device(name: &'static str, width: u32, height: u32, scale: f64, mobile: bool) -> Device {
    Device {
        name,
        width,
        height,
        scale,
        mobile,
    }
}

/// The device named `name`, ignoring case and spaces.
#[must_use]
pub fn device_named(name: &str) -> Option<&'static Device> {
    let wanted = folded(name);
    DEVICES.iter().find(|device| folded(device.name) == wanted)
}

fn folded(name: &str) -> String {
    name.chars()
        .filter(|letter| !letter.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}
