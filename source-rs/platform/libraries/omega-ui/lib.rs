#![no_std]
//! Caller-authored Omega component trees. Apps own models and stable control IDs;
//! the compiled library owns control behavior, text editing, layout and pixels.
//! `Ui::build` repeats a changed view with consumed input edges, up to four passes.
//! Separate instances borrow separate state/packet buffers; no global UI exists.
//! Input is staged then built before the next input edge. Nodes belong to the
//! last completed build, and bounds include control paint overhang.
//!
//! ```
//! use cathedral_omega_ui::{ButtonVariant, Error, TextInput, Ui};
//! fn compose(ui: &mut Ui<'_, '_>, name: &mut TextInput, volume: &mut i32) -> Result<(), Error> {
//!     ui.build(|view| {
//!         view.column(16, 24, true, true)?;
//!         view.heading("Settings")?;
//!         view.text_field(101, name, 260)?;
//!         view.slider(102, volume, 260)?;
//!         if view.button(103, "Reset", ButtonVariant::Secondary, 100, 40)?.clicked {
//!             *volume = 50;
//!         }
//!         view.end()?;
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
//! The first backend uses x86-64 System V and bounded grayscale ASCII text.
//! Presentation uses an app-supplied scoped connection; it grants no authority.
mod native;
#[cfg(target_os = "none")]
mod presentation;
mod text;
mod ui;
#[cfg(target_os = "none")]
pub use native::native_api;
pub use native::{Api, Apply, Query};
#[cfg(target_os = "none")]
pub use presentation::Presenter;
pub use text::TextInput;
pub use ui::{Frame, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Layout,
    Storage,
    Native(u64),
    Rectangle,
    Text,
    StaleNode,
    Capacity,
    Platform(i64),
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub animating: bool,
    pub changed: bool,
    pub focus: i32,
    pub revision: i32,
    pub background: u32,
    pub width: usize,
    pub height: usize,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
/// A frame-local node, usable for paint bounds after its build completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    index: i32,
    generation: u64,
    owner: u64,
}
impl Node {
    pub const NONE: Self = Self {
        index: 0,
        generation: 0,
        owner: 0,
    };
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Response {
    pub node: Node,
    pub clicked: bool,
    pub changed: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
