use id3::{Tag, TagLike};
use std::error::Error;
use std::fs;
use std::path::Path;

pub fn get_mp3_files(dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut mp3_files = Vec::new();
    let path = Path::new(dir);

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && ext == "mp3"
                && let Some(filename) = path.file_name().and_then(|s| s.to_str())
            {
                mp3_files.push(filename.to_string());
            }
        }
    }

    mp3_files.sort();
    Ok(mp3_files)
}

pub fn modify_field(file_path: &str, field: &str, value: &str) -> Result<(), Box<dyn Error>> {
    let mut tag = match Tag::read_from_path(file_path) {
        Ok(tag) => tag,
        Err(_) => Tag::new(),
    };

    match field {
        "Song Name" => {
            tag.set_title(value);
        }
        "Artist" => {
            tag.set_artist(value);
        }
        "Album" => {
            tag.set_album(value);
        }
        "Date" => {
            if let Ok(year) = value.parse() {
                tag.set_date_recorded(year);
            }
        }
        "Track" => {
            if let Ok(track) = value.parse() {
                tag.set_track(track);
            }
        }
        _ => {}
    }

    tag.write_to_path(file_path, id3::Version::Id3v24)?;
    Ok(())
}
