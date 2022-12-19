use anyhow::anyhow;
use anyhow::bail;
use anyhow::Error;
use scopeguard::defer;
use std::ffi::CString;
use std::io::Write;
use std::path::Path;
use stormlib_bindings::SFileCloseArchive;
use stormlib_bindings::SFileCloseFile;
use stormlib_bindings::SFileGetFileSize;
use stormlib_bindings::SFileOpenFileEx;
use stormlib_bindings::SFileReadFile;
use stormlib_bindings::SFileSetLocale;
use stormlib_bindings::ERROR_HANDLE_EOF;
use stormlib_bindings::SFILE_INVALID_SIZE;
use stormlib_bindings::{GetLastError, SFileOpenArchive, HANDLE};
use tracing::{error, instrument};
use uuid::Uuid;

#[instrument(level = "trace", skip_all)]
pub fn get_chk_from_mpq_filename<T: AsRef<Path>>(
    filename: T,
) -> anyhow::Result<Vec<u8>, anyhow::Error> {
    lazy_static::lazy_static! {
        // This is really not the rust way to do things but stormlib_bindings is internally not threadsafe so what we can do.
        static ref LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    }

    let cstr = CString::new(
        filename
            .as_ref()
            .to_str()
            .ok_or(anyhow!("Could not convert filename to str"))?,
    )?;

    let _lock = LOCK.lock().unwrap();
    unsafe {
        let mut mpq_handle = 0 as HANDLE;
        if SFileOpenArchive(cstr.as_ptr(), 0, 0, &mut mpq_handle as *mut _) == false {
            bail!(
                "SFileOpenArchive. GetLastError: {}, filename: {}",
                GetLastError(),
                filename.as_ref().to_string_lossy()
            );
        }

        defer! {
            if SFileCloseArchive(mpq_handle) == false {
                error!(
                    "{:?}",
                    anyhow!(
                        "SFileCloseArchive. GetLastError: {}, filename: {}",
                        GetLastError(),
                        filename.as_ref().to_string_lossy()
                    )
                );
            }
        };

        let try_map_with_locale = |filename: &str, locale| {
            let cstr = CString::new(filename)?;

            SFileSetLocale(locale);
            let mut archive_file_handle = 0 as HANDLE;
            if SFileOpenFileEx(
                mpq_handle,
                cstr.as_ptr(),
                0,
                &mut archive_file_handle as *mut _,
            ) == false
            {
                bail!(
                    "SFileOpenFileEx. GetLastError: {}, filename: {filename}, locale: {locale}",
                    GetLastError()
                );
            }

            defer! {
                if SFileCloseFile(archive_file_handle) == false {
                    error!(
                        "{:?}",
                        anyhow!(
                            "SFileCloseFile. GetLastError: {}, filename: {filename}, locale: {locale}",
                            GetLastError()
                        )
                    );
                }
            };

            let file_size_low;
            let mut file_size_high: u32 = 0;

            file_size_low = SFileGetFileSize(archive_file_handle, &mut file_size_high as *mut _);

            if file_size_low == SFILE_INVALID_SIZE {
                bail!(
                    "SFileGetFileSize. GetLastError: {}, filename: {filename}, locale: {locale}",
                    GetLastError()
                );
            }

            if file_size_high != 0 {
                bail!(
                    "SFileGetFileSize. File size too big. file_size_high: {file_size_high}, file_size_low: {file_size_low}",
                );
            }

            let mut chk_data: Vec<u8> = vec![0; file_size_low as usize];

            let mut size: u32 = 0;
            if SFileReadFile(
                archive_file_handle,
                chk_data.as_mut_ptr() as *mut _,
                chk_data.len() as u32,
                &mut size as *mut _,
                0 as *mut _,
            ) == false
            {
                let last_error = GetLastError();
                if last_error != ERROR_HANDLE_EOF || size == chk_data.len() as u32 {
                    bail!(
                        "SFileReadFile. GetLastError: {}, filename: {filename}, locale: {locale}",
                        last_error,
                    );
                }
            }

            chk_data.resize(size as usize, 0);

            Ok(chk_data)
        };

        let locales = [
            0x404, 0x405, 0x407, 0x409, 0x40a, 0x40c, 0x410, 0x411, 0x412, 0x415, 0x416, 0x419,
            0x809, 0,
        ];

        // PROTECTION: Some maps put fake scenario.chk files at different locales. Try to find the real one by trying a lot of them.
        for locale in locales {
            if let Ok(x) = try_map_with_locale("staredit\\scenario.chk", locale) {
                return Ok(x);
            }
        }

        bail!(
            "Couldn't find scenario.chk the legit way: {}, file: {}",
            GetLastError(),
            filename.as_ref().to_string_lossy(),
        );
    }
}

#[instrument(level = "trace", skip_all)]
pub fn get_chk_from_mpq_in_memory(mpq: &[u8]) -> Result<Vec<u8>, Error> {
    let path = format!("/tmp/{}.scx", Uuid::new_v4().as_simple().to_string());

    let mut file = std::fs::File::create(&path)?;

    defer! {
        if let Err(err) = std::fs::remove_file(&path) {
            error!("{:?}", err);
        }
    }

    file.write(mpq)?;

    file.flush()?;

    get_chk_from_mpq_filename(&path)
}
