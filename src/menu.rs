use crate::platform_impl;

pub struct Menu(pub(crate) platform_impl::Menu);

impl Menu {
    /// Create a new empty window or application menu.
    pub fn new() -> Menu {
        Menu(platform_impl::Menu::new())
    }

    /// Create a new empty context menu.
    pub fn new_for_popup() -> Menu {
        Menu(platform_impl::Menu::new_for_popup())
    }

    /// Consume this `Menu`, returning the platform menu object.
    pub(crate) fn into_inner(self) -> platform_impl::Menu {
        self.0
    }

    /// Add the provided `Menu` as a submenu of self, with the provided title.
    pub fn add_dropdown(&mut self, menu: Menu, text: &str, enabled: bool) {
        self.0.add_dropdown(menu.0, text, enabled)
    }

    /// Add an item to this menu.
    pub fn add_item(&mut self, id: u32, text: &str, selected: Option<bool>, enabled: bool) {
        self.0.add_item(id, text, selected, enabled)
    }

    /// Add a separator to the menu.
    pub fn add_separator(&mut self) {
        self.0.add_separator()
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
