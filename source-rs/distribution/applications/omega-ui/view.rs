//! App-owned model and view: the reusable library knows none of these controls.
use cathedral_omega_ui::{ButtonVariant, Error, Node, Stats, TextInput, Ui};
pub(super) struct Demo {
    pub clicks: i32,
    pub volume: i32,
    pub enabled: bool,
    pub closed: bool,
    pub text: TextInput,
    pub regions: [Node; 6],
    pub region_count: usize,
    dark: bool,
}
impl Default for Demo {
    fn default() -> Self {
        Self {
            clicks: 0,
            volume: 50,
            enabled: true,
            closed: false,
            text: TextInput::default(),
            regions: [Node::NONE; 6],
            region_count: 6,
            dark: true,
        }
    }
}
impl Demo {
    pub fn build(&mut self, ui: &mut Ui<'_, '_>) -> Result<Stats, Error> {
        let width = ui.stats().width as i32 - 64;
        ui.build(|view| {
            view.theme(self.dark)?;
            view.column(16, 32, true, true)?;
            self.regions[0] = view.row(8, 0, true, true)?;
            view.heading("Controls")?;
            view.spacer()?;
            self.closed |= view.button(7, "X", ButtonVariant::Ghost, 32, 32)?.clicked;
            view.end()?;
            self.regions[1] = view.row(8, 0, true, true)?;
            let button_width = (width - 16) / 3;
            if view
                .button(1, "Theme", ButtonVariant::Primary, button_width, 40)?
                .clicked
            {
                self.clicks = self.clicks.saturating_add(1);
                self.dark = !self.dark;
                view.theme(self.dark)?;
            }
            let reset = view
                .button(4, "Reset", ButtonVariant::Secondary, button_width, 40)?
                .clicked;
            let clear = view
                .button(5, "Clear", ButtonVariant::Ghost, button_width, 40)?
                .clicked;
            view.end()?;
            if reset {
                self.volume = 50;
                self.enabled = true;
            }
            if reset || clear {
                self.text.clear();
            }
            self.regions[2] = view.row(12, 0, true, true)?;
            view.label("Volume")?;
            view.spacer()?;
            view.toggle(6, &mut self.enabled)?;
            view.end()?;
            let next = if self.enabled {
                self.regions[3] = view.slider(2, &mut self.volume, width)?.node;
                4
            } else {
                3
            };
            self.regions[next] = view.label("Workspace")?;
            self.regions[next + 1] = view.text_field(3, &mut self.text, width)?.node;
            self.region_count = next + 2;
            view.end()?;
            Ok(())
        })
    }
}
