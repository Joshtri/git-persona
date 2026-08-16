// Notifies the Windows Shell that file associations changed so it flushes its
// icon cache. Called on every app startup to ensure the installed icon is shown
// correctly after an in-place updater run (the NSIS updater replaces the .exe
// but does not invalidate Explorer's icon cache automatically).
//
// SAFETY: SHChangeNotify is a simple shell broadcast with no memory ownership
// contract; passing null item pointers is documented as valid for
// SHCNE_ASSOCCHANGED.
#![allow(unsafe_code)]

pub(crate) fn refresh_icon_cache() {
    use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ID, SHCNF_FLAGS};
    // SHCNE_ASSOCCHANGED (0x08000000) — file-type association changed.
    // SHCNF_IDLIST    (0x00000000) — items are null (not used for this event).
    unsafe {
        SHChangeNotify(SHCNE_ID(0x0800_0000u32), SHCNF_FLAGS(0u32), None, None);
    }
}
