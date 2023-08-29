use std::{error::Error, io::Read};

use flate2::read::GzDecoder;
use tar::Archive;

pub fn read_from_archive(content: &Vec<u8>, file_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let decoder = GzDecoder::new(content.as_slice());
    let mut archive = Archive::new(decoder);

    for entry_result in archive.entries().unwrap() {
        let mut entry = entry_result?;

        if let Some(entry_path) = entry.path()?.to_str() {
            let matchable_path = entry_path.splitn(2, '/').nth(1).unwrap_or("");
            if matchable_path != "" && matchable_path == file_path {
                // Create a buffer to read the contents of the file into
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)?;

                // You can return the buffer or do something else with it
                return Ok(buffer);
            }
        }
    }

    Err("File not found in archive".into())
}
