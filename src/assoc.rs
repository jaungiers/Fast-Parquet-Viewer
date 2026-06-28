//! File-type association for the standalone (non-Store) build.
//!
//! Writes per-user keys under `HKCU\Software\Classes` (no admin needed) so the
//! app appears in Windows' "Open With" list for `.parquet` / `.parq` files. We
//! only advertise a ProgID under each extension's `OpenWithProgids` — we never
//! overwrite the existing default, since Windows 10/11 requires the user to
//! confirm the default handler themselves.
#![cfg(windows)]

use std::io;
use winreg::enums::*;
use winreg::RegKey;

pub const PROGID: &str = "FastParquetViewer.parquet";
const EXTS: [&str; 2] = [".parquet", ".parq"];

/// Register the ProgID and advertise it for our extensions.
pub fn register() -> io::Result<()> {
    let exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // ProgID: friendly name, icon, and the command used to open a file.
    let (progid, _) = hkcu.create_subkey(format!(r"Software\Classes\{PROGID}"))?;
    progid.set_value("", &"Parquet File")?;

    let (icon, _) = hkcu.create_subkey(format!(r"Software\Classes\{PROGID}\DefaultIcon"))?;
    icon.set_value("", &format!("{exe},0"))?;

    let (cmd, _) = hkcu.create_subkey(format!(r"Software\Classes\{PROGID}\shell\open\command"))?;
    cmd.set_value("", &format!("\"{exe}\" \"%1\""))?;

    // Non-destructive: list the ProgID under each extension so it shows in
    // "Open With" without stealing whatever default is already set.
    for ext in EXTS {
        let (k, _) = hkcu.create_subkey(format!(r"Software\Classes\{ext}\OpenWithProgids"))?;
        k.set_value(PROGID, &"")?;
    }

    notify_shell();
    Ok(())
}

/// Remove everything `register` created. Best-effort: missing keys are ignored.
pub fn unregister() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for ext in EXTS {
        if let Ok(k) = hkcu.open_subkey_with_flags(
            format!(r"Software\Classes\{ext}\OpenWithProgids"),
            KEY_SET_VALUE,
        ) {
            let _ = k.delete_value(PROGID);
        }
    }
    let _ = hkcu.delete_subkey_all(format!(r"Software\Classes\{PROGID}"));
    notify_shell();
    Ok(())
}

/// Tell the shell to refresh associations so the change takes effect at once.
fn notify_shell() {
    const SHCNE_ASSOCCHANGED: i32 = 0x0800_0000;
    const SHCNF_IDLIST: u32 = 0x0000;
    #[link(name = "shell32")]
    extern "system" {
        fn SHChangeNotify(
            event: i32,
            flags: u32,
            item1: *const core::ffi::c_void,
            item2: *const core::ffi::c_void,
        );
    }
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED,
            SHCNF_IDLIST,
            core::ptr::null(),
            core::ptr::null(),
        );
    }
}
