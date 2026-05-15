use icrate::ns_string;
use icrate::Foundation::{NSProcessInfo, NSString};
use objc2::rc::Id;
use objc2::runtime::Sel;
use objc2::sel;

use super::appkit::{NSApp, NSEventModifierFlags, NSMenu, NSMenuItem};

pub struct Menu(pub(crate) Id<NSMenu>);

fn make_menu_item(id: u32, text: &str, selected: Option<bool>, enabled: bool) -> Id<NSMenuItem> {
    let item = NSMenuItem::newWithTitle(
        &NSString::from_str(text),
        Some(sel!(handleMenuItem:)),
        ns_string!(""),
    );

    item.setTag(id as isize);

    if !enabled {
        item.setEnabled(false);
    }

    if let Some(true) = selected {
        item.setState(1_isize);
    }
    item
}

impl Menu {
    pub fn new() -> Menu {
        let menu = NSMenu::new();
        // let () = msg_send![menu, setAutoenablesItems: NO];
        Menu(menu)
    }

    pub fn new_for_popup() -> Menu {
        // mac doesn't distinguish between application and context menu types.
        Menu::new()
    }

    pub fn add_dropdown(&mut self, menu: Menu, text: &str, enabled: bool) {
        let menu_item = NSMenuItem::new();
        let title = NSString::from_str(text);
        menu.0.setTitle(&title);
        menu_item.setTitle(&title);
        if !enabled {
            menu_item.setEnabled(false);
        }
        menu_item.setSubmenu(&menu.0);
        self.0.addItem(&menu_item);
    }

    pub fn add_item(&mut self, id: u32, text: &str, selected: Option<bool>, enabled: bool) {
        let menu_item = make_menu_item(id, text, selected, enabled);
        self.0.addItem(&menu_item);
    }

    pub fn add_separator(&mut self) {
        self.0.addItem(&NSMenuItem::separatorItem());
    }
}

struct KeyEquivalent<'a> {
    key: &'a NSString,
    masks: Option<NSEventModifierFlags>,
}

pub fn initialize() {
    let menubar = NSMenu::new();
    let app_menu_item = NSMenuItem::new();
    menubar.addItem(&app_menu_item);

    let app_menu = NSMenu::new();
    let process_name = NSProcessInfo::processInfo().processName();

    // About menu item
    let about_item_title = ns_string!("About ").stringByAppendingString(&process_name);
    let about_item = menu_item(
        &about_item_title,
        Some(sel!(orderFrontStandardAboutPanel:)),
        None,
    );

    // Services menu item
    let services_menu = NSMenu::new();
    let services_item = menu_item(ns_string!("Services"), None, None);
    services_item.setSubmenu(&services_menu);

    // Seperator menu item
    let sep_first = NSMenuItem::separatorItem();

    // Hide application menu item
    let hide_item_title = ns_string!("Hide ").stringByAppendingString(&process_name);
    let hide_item = menu_item(
        &hide_item_title,
        Some(sel!(hide:)),
        Some(KeyEquivalent {
            key: ns_string!("h"),
            masks: None,
        }),
    );

    // Hide other applications menu item
    let hide_others_item_title = ns_string!("Hide Others");
    let hide_others_item = menu_item(
        hide_others_item_title,
        Some(sel!(hideOtherApplications:)),
        Some(KeyEquivalent {
            key: ns_string!("h"),
            masks: Some(
                NSEventModifierFlags::NSAlternateKeyMask | NSEventModifierFlags::NSCommandKeyMask,
            ),
        }),
    );

    // Show applications menu item
    let show_all_item_title = ns_string!("Show All");
    let show_all_item = menu_item(
        show_all_item_title,
        Some(sel!(unhideAllApplications:)),
        None,
    );

    // Seperator menu item
    let sep = NSMenuItem::separatorItem();

    // Quit application menu item
    let quit_item_title = ns_string!("Quit ").stringByAppendingString(&process_name);
    let quit_item = menu_item(
        &quit_item_title,
        Some(sel!(terminate:)),
        Some(KeyEquivalent {
            key: ns_string!("q"),
            masks: None,
        }),
    );

    app_menu.addItem(&about_item);
    app_menu.addItem(&sep_first);
    app_menu.addItem(&services_item);
    app_menu.addItem(&hide_item);
    app_menu.addItem(&hide_others_item);
    app_menu.addItem(&show_all_item);
    app_menu.addItem(&sep);
    app_menu.addItem(&quit_item);
    app_menu_item.setSubmenu(&app_menu);

    let app = NSApp();
    app.setServicesMenu(&services_menu);
    app.setMainMenu(&menubar);
}

fn menu_item(
    title: &NSString,
    selector: Option<Sel>,
    key_equivalent: Option<KeyEquivalent<'_>>,
) -> Id<NSMenuItem> {
    let (key, masks) = match key_equivalent {
        Some(ke) => (ke.key, ke.masks),
        None => (ns_string!(""), None),
    };
    let item = NSMenuItem::newWithTitle(title, selector, key);
    if let Some(masks) = masks {
        item.setKeyEquivalentModifierMask(masks)
    }

    item
}
