use std::ptr::null_mut;

use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError},
    System::Threading::CreateMutexW,
};

const INSTANCE_NAME: &[u16] = &[
    'N' as u16,
    'o' as u16,
    'w' as u16,
    'P' as u16,
    'l' as u16,
    'a' as u16,
    'y' as u16,
    'i' as u16,
    'n' as u16,
    'g' as u16,
    0,
];

pub struct InstanceGuard {
    handle: *mut std::ffi::c_void,
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

pub fn acquire() -> Option<InstanceGuard> {
    unsafe {
        let handle = CreateMutexW(null_mut(), 0, INSTANCE_NAME.as_ptr());

        if handle.is_null() {
            return None;
        }

        if GetLastError() == 183 {
            CloseHandle(handle);
            return None;
        }

        Some(InstanceGuard { handle })
    }
}