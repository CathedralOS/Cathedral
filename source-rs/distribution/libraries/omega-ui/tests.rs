use super::*;
use std::vec;

include!(env!("OMEGA_UI_CONSTANTS"));

#[link(name = "kernel32")]
unsafe extern "system" {
    fn VirtualAlloc(address: *mut u8, bytes: usize, allocation: u32, protection: u32) -> *mut u8;
    fn VirtualProtect(address: *mut u8, bytes: usize, protection: u32, previous: *mut u32) -> i32;
    fn VirtualFree(address: *mut u8, bytes: usize, operation: u32) -> i32;
    fn FlushInstructionCache(process: isize, address: *const u8, bytes: usize) -> i32;
}
struct Code {
    address: *mut u8,
}
impl Code {
    fn new(bytes: &[u8]) -> Self {
        // SAFETY: The test owns this fresh mapping, copies the checked compiled
        // component, then removes write access before invoking any entry.
        unsafe {
            let address = VirtualAlloc(core::ptr::null_mut(), bytes.len(), 0x3000, 4);
            assert!(!address.is_null());
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), address, bytes.len());
            let mut previous = 0;
            assert_ne!(VirtualProtect(address, bytes.len(), 0x20, &mut previous), 0);
            assert_ne!(FlushInstructionCache(-1, address, bytes.len()), 0);
            Self { address }
        }
    }
}
impl Drop for Code {
    fn drop(&mut self) {
        // SAFETY: Every API/instance using this test-owned mapping is gone.
        unsafe {
            assert_ne!(VirtualFree(self.address, 0, 0x8000), 0);
        }
    }
}
fn aligned_bytes(words: &mut [u128]) -> &mut [u8] {
    // SAFETY: u128 has no padding; exclusive slice retains allocation lifetime.
    unsafe { core::slice::from_raw_parts_mut(words.as_mut_ptr().cast(), words.len() * 16) }
}

pub(super) fn with_ui(run: impl FnOnce(&mut Ui<'_, '_>)) {
    let layout = Code::new(LAYOUT_BYTES);
    let frame = Code::new(FRAME_BYTES);
    // SAFETY: Checked pure exports remain RX-mapped until every instance drops.
    let api = unsafe {
        Api::from_entries(
            core::mem::transmute::<*mut u8, Query>(layout.address.add(LAYOUT_ENTRY)),
            core::mem::transmute::<*mut u8, Apply>(frame.address.add(FRAME_ENTRY)),
        )
    };
    let (size, alignment) = api.layout().unwrap();
    assert!(alignment <= 16);
    std::println!(
        "native state: {size} bytes, frame code: {} bytes",
        FRAME_BYTES.len()
    );
    let mut state = vec![0u128; size.div_ceil(16)];
    let mut packet = vec![0u128; 1024];
    assert!(matches!(
        Ui::new(api, &mut [0; 1], aligned_bytes(&mut packet)),
        Err(Error::Storage)
    ));
    let mut ui = Ui::new(api, aligned_bytes(&mut state), aligned_bytes(&mut packet)).unwrap();
    ui.resize(420, 400).unwrap();
    run(&mut ui);
}
struct Preferences {
    account: TextInput,
    workspace: TextInput,
    volume: i32,
    enabled: bool,
    checked: bool,
    clicks: usize,
    nodes: [Node; 7],
}
impl Preferences {
    fn new() -> Self {
        Self {
            account: TextInput::new("Ada").unwrap(),
            workspace: TextInput::new("Cathedral").unwrap(),
            volume: 30,
            enabled: true,
            checked: false,
            clicks: 0,
            nodes: [Node::NONE; 7],
        }
    }
    fn build(&mut self, ui: &mut Ui<'_, '_>) {
        let width = ui.stats().width as i32 - 48;
        ui.build(|view| {
            view.column(12, 24, true, true)?;
            self.nodes[0] = view.heading("Preferences")?;
            self.nodes[1] = view.text_field(101, &mut self.account, width)?.node;
            self.nodes[2] = view.text_field(907, &mut self.workspace, width)?.node;
            view.row(12, 0, true, true)?;
            self.nodes[3] = view.toggle(333, &mut self.enabled)?.node;
            self.nodes[4] = view.checkbox(444, &mut self.checked)?.node;
            let response = view.button(7001, "Apply", ButtonVariant::Primary, 96, 40)?;
            self.nodes[5] = response.node;
            self.clicks += usize::from(response.clicked);
            view.end()?;
            self.nodes[6] = view.slider(59, &mut self.volume, width)?.node;
            view.progress(6000, self.volume, width)?;
            view.end()?;
            Ok(())
        })
        .unwrap();
    }
    fn click(&mut self, ui: &mut Ui<'_, '_>, node: Node) {
        let rectangle = ui.bounds(node).unwrap();
        let (x, y) = (
            (rectangle.x + rectangle.width / 2) as i32,
            (rectangle.y + rectangle.height / 2) as i32,
        );
        ui.pointer(x, y, true, true, false).unwrap();
        self.build(ui);
        ui.pointer(x, y, false, false, true).unwrap();
        self.build(ui);
    }
    fn key(&mut self, ui: &mut Ui<'_, '_>, key: i32, character: i32, control: bool, shift: bool) {
        ui.key(key, character, control, shift).unwrap();
        self.build(ui);
    }
}
#[test]
fn caller_authored_controls_edit_independent_fields_and_consume_clicks_once() {
    with_ui(|ui| {
        let mut app = Preferences::new();
        app.build(ui);
        assert!(ui.bounds(app.nodes[2]).unwrap().y > ui.bounds(app.nodes[1]).unwrap().y);
        app.click(ui, app.nodes[2]);
        app.key(ui, 65, 0, true, false);
        app.key(ui, 0, 90, false, false);
        assert_eq!(app.workspace.as_str(), "Z");
        assert_eq!(app.account.as_str(), "Ada");
        app.click(ui, app.nodes[1]);
        app.key(ui, 65, 0, true, false);
        app.key(ui, 0, 87, false, false);
        app.key(ui, 0, 105, false, false);
        app.key(ui, 37, 0, false, true);
        app.key(ui, 0, 120, false, false);
        assert_eq!(app.account.as_str(), "Wx");
        assert_eq!(app.workspace.as_str(), "Z");
        app.click(ui, app.nodes[3]);
        assert!(!app.enabled);
        app.click(ui, app.nodes[4]);
        assert!(app.checked);
        app.click(ui, app.nodes[5]);
        assert_eq!(app.clicks, 1);
        app.build(ui);
        assert_eq!(app.clicks, 1);
        let slider = ui.bounds(app.nodes[6]).unwrap();
        ui.pointer(
            (slider.x + slider.width * 3 / 4) as i32,
            (slider.y + slider.height / 2) as i32,
            true,
            true,
            false,
        )
        .unwrap();
        app.build(ui);
        assert!((70..=80).contains(&app.volume));
        ui.pointer(-1, -1, false, false, true).unwrap();
        app.build(ui);
        app.click(ui, app.nodes[1]);
        app.key(ui, 65, 0, true, false);
        for _ in 0..140 {
            app.key(ui, 0, 97, false, false);
        }
        assert_eq!(app.account.as_str().len(), 128);
        let field = ui.bounds(app.nodes[1]).unwrap();
        assert!(ui.row(field.y + 20).unwrap().len() == 420);
        app.key(ui, 65, 0, true, false);
        app.key(ui, 8, 0, false, false);
        assert_eq!(app.account.as_str(), "");
        assert_eq!(app.workspace.as_str(), "Z");
        let mut shades = std::collections::BTreeSet::new();
        let title = ui.bounds(app.nodes[0]).unwrap();
        for y in title.y..title.y + title.height {
            shades.extend(
                ui.row(y).unwrap()[title.x..title.x + title.width]
                    .iter()
                    .copied(),
            );
        }
        assert!(
            shades.len() > 6,
            "shared glyph coverage must remain antialiased"
        );
    });
}
#[test]
fn another_app_can_use_a_different_tree_and_native_instances_are_isolated() {
    with_ui(|first| {
        let mut preferences = Preferences::new();
        preferences.build(first);
        let first_node = preferences.nodes[1];
        with_ui(|second| {
            second.resize(600, 180).unwrap();
            let mut counter = 7;
            let mut field = TextInput::new("Different app").unwrap();
            let mut node = Node::NONE;
            second
                .build(|view| {
                    view.row(16, 20, true, true)?;
                    view.label("Counter")?;
                    node = view.text_field(101, &mut field, 180)?.node;
                    view.slider(22222, &mut counter, 120)?;
                    view.end()?;
                    Ok(())
                })
                .unwrap();
            assert_eq!(field.as_str(), "Different app");
            assert_eq!(counter, 7);
            assert_eq!(second.bounds(first_node), Err(Error::StaleNode));
            assert_eq!(first.bounds(node), Err(Error::StaleNode));
            assert!(second.bounds(node).unwrap().x > 20);
            assert_eq!(first.stats().width, 420);
            assert_eq!(preferences.account.as_str(), "Ada");
        });
    });
}
#[test]
fn rejected_frames_do_not_publish_and_the_instance_recovers() {
    with_ui(|ui| {
        let mut node = Node::NONE;
        ui.build(|view| {
            node = view.label("Old frame")?;
            Ok(())
        })
        .unwrap();
        let old_node = node;
        let duplicate = ui.build(|view| {
            view.button(9, "One", ButtonVariant::Primary, 80, 40)?;
            view.button(9, "Two", ButtonVariant::Primary, 80, 40)?;
            Ok(())
        });
        assert_eq!(duplicate, Err(Error::Native(5)));
        assert!(ui.row(0).is_err());
        assert_eq!(ui.bounds(old_node), Err(Error::StaleNode));
        assert_eq!(
            ui.build(|view| {
                view.column(8, 8, true, true)?;
                Ok(())
            }),
            Err(Error::Native(4))
        );
        assert_eq!(
            ui.build(|view| {
                view.label("non-ASCII \u{03a9}")?;
                Ok(())
            }),
            Err(Error::Text)
        );
        assert!(
            ui.build(|view| {
                view.label("Recovered")?;
                Ok(())
            })
            .is_ok()
        );
        assert!(ui.row(0).is_ok());
        assert_eq!(ui.resize(3841, 200), Err(Error::Rectangle));
        assert_eq!(ui.key(0, 0x1000, false, false), Err(Error::Text));
    });
}
#[test]
fn motion_and_dynamic_geometry_use_the_same_shared_controls() {
    with_ui(|ui| {
        let mut node = Node::NONE;
        let mut build = |ui: &mut Ui<'_, '_>| {
            ui.build(|view| {
                view.column(8, 24, true, true)?;
                node = view
                    .button(8080, "Launch", ButtonVariant::Secondary, 120, 40)?
                    .node;
                view.end()?;
                Ok(())
            })
            .unwrap();
            node
        };
        let node = build(ui);
        let rectangle = ui.bounds(node).unwrap();
        let (x, y) = (rectangle.x + 8, rectangle.y + 8);
        let before = ui.row(y).unwrap()[x];
        ui.pointer(x as i32, y as i32, false, false, false).unwrap();
        build(ui);
        assert!(ui.stats().animating);
        ui.tick(50).unwrap();
        build(ui);
        assert_ne!(before, ui.row(y).unwrap()[x]);
        ui.tick(500).unwrap();
        build(ui);
        assert!(!ui.stats().animating);
        ui.resize(240, 240).unwrap();
        let next = build(ui);
        assert_eq!(ui.bounds(node), Err(Error::StaleNode));
        assert!(ui.bounds(next).is_ok());
    });
}

#[test]
fn caller_labels_clip_to_their_control_bounds() {
    with_ui(|ui| {
        ui.resize(240, 120).unwrap();
        let mut button = Node::NONE;
        ui.build(|view| {
            view.column(8, 20, true, true)?;
            view.row(8, 0, true, true)?;
            button = view
                .button(
                    811,
                    "A label much longer than its button",
                    ButtonVariant::Primary,
                    80,
                    40,
                )?
                .node;
            view.end()?;
            view.end()?;
            Ok(())
        })
        .unwrap();
        let rectangle = ui.bounds(button).unwrap();
        assert_eq!(rectangle.width, 80);
        let background = ui.stats().background;
        let row = ui.row(rectangle.y + 20).unwrap();
        assert!(
            row[rectangle.x + rectangle.width + 2..]
                .iter()
                .all(|pixel| *pixel as u32 == background),
            "label pixels escaped the control's paint bounds"
        );
    });
}
