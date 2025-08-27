pub mod config;
pub mod database;
pub mod downloader;
pub mod error;

pub use database::DatabaseManager;
pub use error::{Error, Result};

#[repr(C)]
pub struct GladeDatabase {
    manager: DatabaseManager,
}

#[no_mangle]
pub extern "C" fn glade_new() -> *mut GladeDatabase {
    match DatabaseManager::new() {
        Ok(manager) => {
            let db = Box::new(GladeDatabase { manager });
            Box::into_raw(db)
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn glade_free(ptr: *mut GladeDatabase) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn glade_download_database(
    ptr: *mut GladeDatabase,
    db_name: *const std::os::raw::c_char,
    genome_version: *const std::os::raw::c_char,
) -> std::os::raw::c_int {
    if ptr.is_null() || db_name.is_null() || genome_version.is_null() {
        return -1;
    }

    unsafe {
        let db_name_str = match std::ffi::CStr::from_ptr(db_name).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let genome_version_str = match std::ffi::CStr::from_ptr(genome_version).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let database = &(*ptr).manager;

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return -1,
        };

        match runtime.block_on(database.download_database(db_name_str, genome_version_str)) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}
