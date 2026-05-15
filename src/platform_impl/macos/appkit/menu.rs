use icrate::Foundation::{NSObject, NSPoint, NSString};
use objc2::ffi::BOOL;
use objc2::rc::Id;
use objc2::{extern_class, extern_methods, mutability, ClassType};

use super::{NSMenuItem, NSView};

extern_class!(
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub(crate) struct NSMenu;

    unsafe impl ClassType for NSMenu {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
    }
);

unsafe impl Send for NSMenu {}
unsafe impl Sync for NSMenu {}

extern_methods!(
    unsafe impl NSMenu {
        #[method_id(new)]
        pub fn new() -> Id<Self>;

        #[method(addItem:)]
        pub fn addItem(&self, item: &NSMenuItem);

        #[method(setTitle:)]
        pub fn setTitle(&self, title: &NSString);

        #[method(popUpMenuPositioningItem:atLocation:inView:)]
        pub fn popUpMenuPositioningItem(
            &self,
            item: *mut NSMenuItem,
            location: NSPoint,
            inView: *mut NSView,
        ) -> BOOL;
    }
);
