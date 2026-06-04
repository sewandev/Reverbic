#![cfg(target_os = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use windows::core::{implement, Result, PCWSTR};
use windows::Win32::Media::Audio::{
    EDataFlow, ERole, IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl,
    MMDeviceEnumerator, DEVICE_STATE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY;

#[implement(IMMNotificationClient)]
struct DeviceCallback {
    changed: Arc<AtomicBool>,
}

impl IMMNotificationClient_Impl for DeviceCallback_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }

    fn OnDeviceAdded(&self, _: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _: &PCWSTR) -> Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }

    fn OnDefaultDeviceChanged(&self, _: EDataFlow, _: ERole, _: &PCWSTR) -> Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}

pub fn spawn_monitor(changed: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("audio-device-monitor".into())
        .spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }
            if let Err(e) = register_and_block(changed) {
                tracing::warn!("audio-device-monitor: {e}");
            }
        })
        .expect("fallo al iniciar audio-device-monitor");
}

fn register_and_block(changed: Arc<AtomicBool>) -> windows::core::Result<()> {
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let callback: IMMNotificationClient = DeviceCallback { changed }.into();
    unsafe { enumerator.RegisterEndpointNotificationCallback(&callback)? };
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
