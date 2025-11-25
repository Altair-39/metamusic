use crate::functions::*;

use id3::{Tag, TagLike};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

pub struct App {
    files: Vec<String>,
    selected_file: usize,
    fields: Vec<String>,
    selected_field: usize,
    input_buffer: String,
    current_field: Option<String>,
    current_file: String,
    mode: Mode,
    message: String,
    pub album_art_cache: HashMap<String, Arc<Mutex<StatefulProtocol>>>,
    pub image_picker: Picker,
}

#[derive(Clone)]
pub struct TagInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub track: String,
}

#[derive(PartialEq)]
pub enum Mode {
    FileSelection,
    FieldSelection,
    Editing,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let files = get_mp3_files(".")?;
        // Initialize the image picker
        let image_picker = Picker::from_fontsize((10, 24));

        Ok(App {
            files: files.clone(),
            selected_file: 0,
            fields: vec![
                "Song Name".to_string(),
                "Artist".to_string(),
                "Album".to_string(),
                "Date".to_string(),
                "Track".to_string(),
            ],
            selected_field: 0,
            input_buffer: String::new(),
            current_field: None,
            current_file: files.first().cloned().unwrap_or_default(),
            mode: Mode::FileSelection,
            message: String::from("Select a file to edit"),
            album_art_cache: HashMap::new(),
            image_picker,
        })
    }

    pub fn load_album_art(&mut self, filename: &str) -> Option<Arc<Mutex<StatefulProtocol>>> {
        // Check cache first
        if let Some(cached) = self.album_art_cache.get(filename) {
            return Some(cached.clone());
        }

        // Extract album art from MP3 file
        if let Some(art_data) = self.extract_album_art_bytes(filename) {
            // Try to decode the image
            if let Ok(dyn_img) = image::load_from_memory(&art_data) {
                // Create protocol for rendering
                let protocol = self.image_picker.new_resize_protocol(dyn_img);
                let protocol_arc = Arc::new(Mutex::new(protocol));

                self.album_art_cache
                    .insert(filename.to_string(), protocol_arc.clone());
                return Some(protocol_arc);
            }
        }

        None
    }

    fn extract_album_art_bytes(&self, filename: &str) -> Option<Vec<u8>> {
        match Tag::read_from_path(filename) {
            Ok(tag) => {
                if let Some(picture) = tag.pictures().next() {
                    return Some(picture.data.clone());
                }
                None
            }
            Err(_) => None,
        }
    }
    pub fn has_album_art(&self, filename: &str) -> bool {
        self.album_art_cache.contains_key(filename) || {
            match Tag::read_from_path(filename) {
                Ok(tag) => tag.pictures().next().is_some(),
                Err(_) => false,
            }
        }
    }
    pub fn next_item(&mut self) {
        match self.mode {
            Mode::FileSelection => {
                if !self.files.is_empty() {
                    self.selected_file = (self.selected_file + 1) % self.files.len();
                    self.current_file = self.files[self.selected_file].clone();
                }
            }
            Mode::FieldSelection => {
                self.selected_field = (self.selected_field + 1) % self.fields.len();
            }
            _ => {}
        }
    }

    pub fn previous_item(&mut self) {
        match self.mode {
            Mode::FileSelection => {
                if !self.files.is_empty() {
                    if self.selected_file > 0 {
                        self.selected_file -= 1;
                    } else {
                        self.selected_file = self.files.len() - 1;
                    }
                    self.current_file = self.files[self.selected_file].clone();
                }
            }
            Mode::FieldSelection => {
                if self.selected_field > 0 {
                    self.selected_field -= 1;
                } else {
                    self.selected_field = self.fields.len() - 1;
                }
            }
            _ => {}
        }
    }

    pub fn start_field_selection(&mut self) {
        if !self.files.is_empty() {
            self.mode = Mode::FieldSelection;
            self.message = format!("Editing: {}", self.current_file);
        }
    }

    pub fn start_editing(&mut self) {
        self.mode = Mode::Editing;
        self.input_buffer.clear();
        self.current_field = Some(self.fields[self.selected_field].clone());

        if let Ok(tag) = Tag::read_from_path(&self.current_file) {
            match self.fields[self.selected_field].as_str() {
                "Song Name" => self.input_buffer = tag.title().unwrap_or("").to_string(),
                "Artist" => self.input_buffer = tag.artist().unwrap_or("").to_string(),
                "Album" => self.input_buffer = tag.album().unwrap_or("").to_string(),
                "Date" => self.input_buffer = tag.year().map(|y| y.to_string()).unwrap_or_default(),
                "Track" => {
                    self.input_buffer = tag.track().map(|t| t.to_string()).unwrap_or_default()
                }
                _ => {}
            }
        }
    }

    pub fn finish_editing(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(field) = &self.current_field {
            let result = modify_field(&self.current_file, field, &self.input_buffer);
            match result {
                Ok(_) => {
                    self.message = format!("✓ Updated {} to '{}'", field, self.input_buffer);
                }
                Err(e) => {
                    self.message = format!("✗ Error: {}", e);
                }
            }
        }
        self.mode = Mode::FieldSelection;
        self.current_field = None;
        Ok(())
    }

    pub fn cancel_editing(&mut self) {
        self.mode = Mode::FieldSelection;
        self.current_field = None;
        self.message = "Edit cancelled".to_string();
    }

    pub fn back_to_files(&mut self) {
        self.mode = Mode::FileSelection;
        self.message = "Select a file to edit".to_string();
    }

    pub fn files(&self) -> &[String] {
        &self.files
    }

    pub fn selected_file(&self) -> usize {
        self.selected_file
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    pub fn selected_field(&self) -> usize {
        self.selected_field
    }

    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn current_field(&self) -> Option<&String> {
        self.current_field.as_ref()
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    pub fn push_to_buffer(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn pop_from_buffer(&mut self) {
        self.input_buffer.pop();
    }

    pub fn tags_for_file(&self, filename: &str) -> Option<TagInfo> {
        match Tag::read_from_path(filename) {
            Ok(tag) => Some(TagInfo {
                title: tag
                    .title()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                artist: tag
                    .artist()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                album: tag
                    .album()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                year: tag
                    .date_recorded()
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                track: tag
                    .track()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
            }),
            Err(_) => None,
        }
    }
}
