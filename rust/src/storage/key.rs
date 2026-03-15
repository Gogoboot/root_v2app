// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — storage/key.rs
// ═══════════════════════════════════════════════════════════

use std::path::PathBuf;
use std::fs;
use rand::RngCore;

pub type Salt = [u8; 16];

fn get_salt_path() -> Option<PathBuf> {
    let app_dir = dirs::data_local_dir()?;
    let root_dir = app_dir.join("root_app");
    fs::create_dir_all(&root_dir).ok()?;
    Some(root_dir.join("salt.bin"))
}

pub fn load_salt() -> Option<Salt> {
    let path = get_salt_path()?;
    if !path.exists() { return None; }
    let bytes = fs::read(&path).ok()?;
    if bytes.len() != 16 { return None; }
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&bytes);
    Some(salt)
}

pub fn save_salt(salt: &Salt) -> Result<(), std::io::Error> {
    let path = get_salt_path().ok_or(
        std::io::Error::new(std::io::ErrorKind::Other, "Cannot get salt path")
    )?;
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::write(&path, salt.as_slice())?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(windows)] {
        fs::write(&path, salt.as_slice())?;
    }
    Ok(())
}

pub fn get_or_generate_salt() -> Result<Salt, std::io::Error> {
    if let Some(salt) = load_salt() { return Ok(salt); }
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    save_salt(&salt)?;
    Ok(salt)
}
