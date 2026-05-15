use std::{collections::HashMap, ptr::null};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateMenu, CreatePopupMenu, DestroyMenu, ACCEL, HMENU, MF_CHECKED, MF_GRAYED,
    MF_POPUP, MF_SEPARATOR, MF_STRING,
};

/// A menu object, which can be either a top-level menubar or a
/// submenu.
pub struct Menu {
    hmenu: HMENU,
    accels: HashMap<u32, ACCEL>,
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            DestroyMenu(self.hmenu);
        }
    }
}

impl Menu {
    /// Create a new menu for a window.
    pub fn new() -> Menu {
        unsafe {
            let hmenu = CreateMenu();
            Menu {
                hmenu,
                accels: HashMap::default(),
            }
        }
    }

    /// Create a new popup (context / right-click) menu.
    pub fn new_for_popup() -> Menu {
        unsafe {
            let hmenu = CreatePopupMenu();
            Menu {
                hmenu,
                accels: HashMap::default(),
            }
        }
    }

    pub fn into_hmenu(self) -> HMENU {
        let hmenu = self.hmenu;
        std::mem::forget(self);
        hmenu
    }

    /// Add a dropdown menu. This takes the menu by ownership, but we'll
    /// probably want to change that so we can manipulate it later.
    ///
    /// The `text` field has all the fun behavior of winapi CreateMenu.
    pub fn add_dropdown(&mut self, mut menu: Menu, text: &str, enabled: bool) {
        let child_accels = std::mem::take(&mut menu.accels);
        self.accels.extend(child_accels);

        unsafe {
            let mut flags = MF_POPUP;
            if !enabled {
                flags |= MF_GRAYED;
            }
            let text = text
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>();
            AppendMenuW(self.hmenu, flags, menu.into_hmenu() as usize, text.as_ptr());
        }
    }

    /// Add an item to the menu.
    pub fn add_item(&mut self, id: u32, text: &str, selected: Option<bool>, enabled: bool) {
        unsafe {
            let mut flags = MF_STRING;
            if !enabled {
                flags |= MF_GRAYED;
            }
            if let Some(true) = selected {
                flags |= MF_CHECKED;
            }
            let text = text
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>();
            AppendMenuW(self.hmenu, flags, id as usize, text.as_ptr());
        }
    }

    /// Add a separator to the menu.
    pub fn add_separator(&mut self) {
        unsafe {
            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, null());
        }
    }

    /// Get the accels table
    pub fn accels(&self) -> Option<Vec<ACCEL>> {
        if self.accels.is_empty() {
            return None;
        }
        Some(self.accels.values().cloned().collect())
    }
}
