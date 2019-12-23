use std::sync::{
    collections::HashMap,
    ffi::OsStr,
    iter::once,
    mpsc::{Receiver, Sender},
    os::windows::OsStrExt,
    Arc, Mutex,
};
use winapi::{
    shared::{
        minwindef::{LOWORD, LPARAM, LRESULT, UINT, WPARAM},
        windef::{HWND, POINT},
    },
    um::winuser::{
        AppendMenuW, CreatePopupMenu, DefWindowProcW, DispatchMessage, GetMessage,
        RegisterClassExW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, IDI_APPLICATION,
        MF_STRING, MSG, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER, WNDCLASSEXW,
    },
};

const BUTTON_PRESS: UINT = WM_USER + 1;

struct MenuItem {
    label: Vec<u16>,
    f: Box<dyn Fn() -> ()>,
}
struct WindowsSysBar {
    name: String,
    callbacks: Vec<MenuItem>,
}

impl crate::Bar for WindowsSysBar {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            callbacks: Vec::new(),
        }
    }
    fn add_item(&mut self, label: &str, cbs: Box<dyn Fn() -> ()>) {
        let encoded_label = to_win_string(label);
        self.callbacks.push(MenuItem {
            label: encoded_label,
            f: cbs,
        });
    }

    fn add_quit_item(&mut self) {
        self.add_item("Quit", Box::new(|| {}));
    }

    fn display(&mut self) {
        let mut instance = unsafe { GetModuleHandle(0 as _) };
        let mut hWnd = unsafe { InitInstance(&mut instance as _, FALSE) };
        let bits = self.get_icon();
        let mut icon = unsafe { CreateIcon(instance, 8, 8, 1, 1, &bits as _, &bits as _) };
        let mut icon_data: NOTIFYICONDATA = std::mem::zeroed();
        icon_data.cbSize = std::mem::size_of::<NOTIFYICONDATA>();
        icon_data.hWnd = hWnd;
        icon_data.uID = 99;
        icon_data.uCallbackMessage = BUTTON_PRESS;
        icon_data.hIcon = icon;
        icon_data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        icon_data.szTip = to_win_string(&self.name);
        unsafe {
            Shell_NotifyIcon(NIM_ADD, &icon_data as _);
        }
        while self.tick(hWnd) >= 0 {}
        unsafe {
            Shell_NotifyIcon(NIM_DELETE, &icon_data as _);
            PostQuitMessage(0);
        }
    }
}

impl WindowsSysBar {
    fn get_icon(&self) -> [u8; 8] {
        use font8x8::{UnicodeFonts, BASIC_FONTS};
        if let Some(c) = self.name.chars().next() {
            if let Some(bits) = BASIC_FONTS.get(c) {
                bits
            } else {
                [0; u8]
            }
        } else {
            [0; u8]
        }
    }

    fn register(instance: HINSTANCE) {
        let mut wcex: WNDCLASSEXW = std::mem::zeroed();

        wcex.cbSize = std::mem::size_of::<WNDCLASSEXW>() as _;

        wcex.style = CS_HREDRAW | CS_VREDRAW;
        wcex.lpfnWndProc = 0 as _;
        wcex.cbClsExtra = 0;
        wcex.cbWndExtra = 0;
        wcex.hInstance = instance;
        wcex.hIcon = unsafe { LoadIcon(0 as _, IDI_APPLICATION) };
        wcex.hCursor = unsafe { LoadCursor(0 as _, IDC_ARROW) };
        wcex.hbrBackground = COLOR_WINDOW + 1 as _;
        wcex.lpszMenuName = 0;
        wcex.lpszClassName = &to_win_string("SysBar") as _;
        wcex.hIconSm = unsafe { LoadIcon(0 as _, IDI_APPLICATION) };

        unsafe { RegisterClassExW(&wcex) };
    }

    fn tick(&self, hWnd: HWND) -> LRESULT {
        let mut message: MSG = std::mem::zeroed();

        if unsafe { GetMessage(&mut message as _, 0, 0, 0) } > 0 {
            return 0;
        }

        match (message.message) {
            WM_COMMAND => {
                let idx = unsafe { LOWORD(message.wparam) } as usize - 1001;
                if let Some(item) = self.callbacks.get(idx) {
                    if item.label == "Quit" {
                        return -1;
                    }
                    item.f()
                }
                0
            }
            WM_DESTROY => -1,
            BUTTON_PRESS => match lParam {
                WM_RBUTTONUP => self.show_menu(),
                WM_LBUTTONUP => 0,
                _ => Self::default_message_handler(&message),
            },
            _ => Self::default_message_handler(&message),
        }
    }

    fn show_menu(&self, hWnd: HWND) -> LRESULT {
        let mut menu = unsafe { CreatePopupMenu() };
        let mut p: POINT = std::mem::zeroed();
        unsafe {
            GetCursorPos(&mut p);
        }
        for (idx, item) in self.callbacks.enumerate() {
            unsafe { AppendMenuW(&mut menu as _, MF_STRING, idx + 1001 as _, &item.label as _) }
        }
        unsafe {
            TrackPopupMenu(
                &mut menu as _,
                TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                p.x,
                p.y,
                0,
                hWnd,
                0 as _,
            );
        }
        0
    }

    fn default_message_handler(msg: &MSG) -> LRESULT {
        unsafe {
            TranslateMessage(msg as _);
            DispatchMessage(msg as _);
        }
        0
    }
}

fn to_win_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}
