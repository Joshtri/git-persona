use crate::domain::ports::GitConfigBackend;
use crate::error::AppError;
use gix_config::File;

pub(crate) struct GixGitConfig;

fn home_gitconfig() -> Result<std::path::PathBuf, AppError> {
    dirs_next::home_dir()
        .map(|h| h.join(".gitconfig"))
        .ok_or_else(|| AppError::GitConfig("could not find home directory".into()))
}

fn open(path: &std::path::Path) -> Result<File<'static>, AppError> {
    if path.exists() {
        gix_config::File::from_path_no_includes(path.to_path_buf(), gix_config::Source::User)
            .map_err(|e| AppError::GitConfig(e.to_string()))
    } else {
        let meta = gix_config::file::Metadata::from(gix_config::Source::User);
        Ok(gix_config::File::new(meta))
    }
}

fn get_value(section: &str, key: &str) -> Result<Option<String>, AppError> {
    let path = home_gitconfig()?;
    match open(&path) {
        Ok(f) => Ok(f
            .raw_value_by(section, None, key)
            .ok()
            .map(|v| String::from_utf8_lossy(v.as_ref()).into_owned())),
        Err(_) => Ok(None),
    }
}

fn set_value(section: &str, key: &str, value: Option<&str>) -> Result<(), AppError> {
    let path = home_gitconfig()?;
    // Owned key avoids lifetime conflict with File<'static>'s invariant 'event param.
    let value_name: gix_config::parse::section::ValueName<'static> = key
        .to_string()
        .try_into()
        .map_err(|_| AppError::GitConfig(format!("invalid key: {key}")))?;

    let mut f = open(&path)?;

    let mut sec = f
        .section_mut_or_create_new(section, None)
        .map_err(|e| AppError::GitConfig(e.to_string()))?;

    while sec.remove(key).is_some() {}

    if let Some(v) = value {
        let normalized = gix_config::value::normalize_bstr(v);
        sec.push(value_name, Some(normalized.as_ref()));
    }

    drop(sec);

    let serialized = f.to_string();
    std::fs::write(&path, serialized).map_err(|e| AppError::GitConfig(e.to_string()))
}

impl GitConfigBackend for GixGitConfig {
    fn get_global_name(&self) -> Result<Option<String>, AppError> {
        get_value("user", "name")
    }

    fn get_global_email(&self) -> Result<Option<String>, AppError> {
        get_value("user", "email")
    }

    fn get_global_signing_key(&self) -> Result<Option<String>, AppError> {
        get_value("user", "signingkey")
    }

    fn set_global_name(&self, name: &str) -> Result<(), AppError> {
        set_value("user", "name", Some(name))
    }

    fn set_global_email(&self, email: &str) -> Result<(), AppError> {
        set_value("user", "email", Some(email))
    }

    fn set_global_signing_key(&self, key: Option<&str>) -> Result<(), AppError> {
        set_value("user", "signingkey", key)
    }
}
