use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
}

pub fn list_dirs(path: &str) -> Result<Vec<DirEntry>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return Err("not implemented".into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut entries: Vec<DirEntry> = std::fs::read_dir(path)
            .map_err(|e| format!("{e}"))?
            .filter_map(|r| r.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .map(|e| {
                let path = e.path().to_string_lossy().into_owned();
                let name = e.file_name().to_string_lossy().into_owned();
                DirEntry { name, path }
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }
}

pub fn get_home() -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return Err("not implemented".into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("HOME").map_err(|e| format!("{e}"))
    }
}
