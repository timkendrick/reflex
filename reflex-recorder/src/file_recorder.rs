// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex_utils::{event::EventSink, load_messages, FileWriter, FileWriterFormat};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

#[derive(Debug)]
pub enum FileRecorderLoadError {
    Load(std::io::Error),
    Deserialize(Box<dyn std::error::Error + Send + Sync>),
}
impl std::error::Error for FileRecorderLoadError {}
impl std::fmt::Display for FileRecorderLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(err) => write!(f, "Failed to load session recording: {}", err),
            Self::Deserialize(err) => write!(f, "Failed to deserialize session recording: {}", err),
        }
    }
}

pub struct FileRecorder<TEvent>(FileWriter<TEvent>)
where
    TEvent: Send + 'static;
impl<TEvent> FileRecorder<TEvent>
where
    TEvent: Send + 'static,
{
    pub fn create(format: FileWriterFormat, path: &Path) -> Result<Self, String>
    where
        TEvent: Clone + Serialize,
    {
        File::create(path)
            .map_err(|err| {
                format!(
                    "Failed to create session recorder output file: {} ({})",
                    path.to_string_lossy(),
                    err
                )
            })
            .map(|output_file| Self::from(FileWriter::new(format, output_file)))
    }
    pub fn load(format: FileWriterFormat, path: &Path) -> Result<Vec<TEvent>, FileRecorderLoadError>
    where
        TEvent: for<'de> Deserialize<'de>,
    {
        let input_file = File::open(path).map_err(FileRecorderLoadError::Load)?;
        load_messages(format, &input_file).map_err(FileRecorderLoadError::Deserialize)
    }
}
impl<TEvent> EventSink for FileRecorder<TEvent>
where
    TEvent: Clone + Serialize + Send + 'static,
{
    type Event = TEvent;
    fn emit(&mut self, event: &Self::Event) {
        let Self(writer) = self;
        // TODO: handle errors when recording operations
        let _ = writer.write(event.clone());
    }
}
impl<TEvent> From<FileWriter<TEvent>> for FileRecorder<TEvent>
where
    TEvent: Clone + Serialize + Send + 'static,
{
    fn from(writer: FileWriter<TEvent>) -> Self {
        Self(writer)
    }
}
