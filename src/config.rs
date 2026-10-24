use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

const DEFAULT_SELECTED_SOURCE: &str = "Spotify.exe";

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    selected_source: String,
}

fn config_path() -> io::Result<PathBuf> {
    let app_data = env::var_os("APPDATA").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "APPDATA environment variable is unavailable",
        )
    })?;

    Ok(PathBuf::from(app_data)
        .join("NowPlaying")
        .join("config.json"))
}

pub fn load_selected_source() -> String {
    let result = config_path().and_then(|path| {
        let contents = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&contents)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        let source = config.selected_source.trim();

        if source.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Selected source cannot be empty",
            ));
        }

        Ok(source.to_string())
    });

    match result {
        Ok(source) => {
            println!("Loaded media source preference: {source}");
            source
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            println!("No saved configuration found; using default media source.");
            DEFAULT_SELECTED_SOURCE.to_string()
        }
        Err(error) => {
            eprintln!("Could not load configuration; using default media source: {error}");
            DEFAULT_SELECTED_SOURCE.to_string()
        }
    }
}

pub fn save_selected_source(source: &str) -> io::Result<()> {
    let source = source.trim();

    if source.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Selected source cannot be empty",
        ));
    }

    let path = config_path()?;

    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Configuration path has no parent directory",
        )
    })?;

    fs::create_dir_all(parent)?;

    let config = AppConfig {
        selected_source: source.to_string(),
    };

    let contents = serde_json::to_string_pretty(&config).map_err(io::Error::other)?;

    fs::write(path, contents)
}
