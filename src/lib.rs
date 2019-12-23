#[cfg(windows)]
extern crate font8x8;

#[cfg(target_os = "macos")]
mod mac_os;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(windows)]
mod windows;

pub struct Sysbar<T = SysbarImpl>(T);

impl Sysbar<SysbarImpl> {
    pub fn new(name: &str) -> Self {
        Sysbar(SysbarImpl::new(name))
    }

    pub fn add_item(&mut self, label: &str, cbs: Box<dyn Fn() -> ()>) {
        self.0.add_item(label, cbs)
    }

    pub fn add_quit_item(&mut self, label: &str) {
        self.0.add_quit_item(label)
    }

    pub fn display(&mut self) {
        self.0.display()
    }
}

#[cfg(target_os = "macos")]
type SysbarImpl = mac_os::MacOsSysbar;

pub trait Bar {
    fn new(name: &str) -> Self;
    fn add_item(&mut self, label: &str, action: Box<dyn Fn()>);
    fn add_quit_item(&mut self, label: &str);
    fn display(&mut self);
}
