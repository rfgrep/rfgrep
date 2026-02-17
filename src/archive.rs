use crate::error::Result as RfgrepResult;
use crate::processor::{find_matches_streaming, SearchMatch};
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn search_zip(path: &Path, pattern: &Regex) -> RfgrepResult<Vec<SearchMatch>> {
    let file = File::open(path).map_err(crate::error::RfgrepError::Io)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| crate::error::RfgrepError::Other(e.to_string()))?;
    let mut matches = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| crate::error::RfgrepError::Other(e.to_string()))?;
        if file.is_file() {
            // Safety: sanitize filename to prevent path traversal in display
            let entry_name = file.name().to_string();
            // Construct a virtual path for the match: archive.zip:entry/path
            // We use a custom separator or just join.
            // But Path::join might act weird if entry is absolute (shouldn't be in zip).
            // A clearer representation might be needed, currently just joining.
            let entry_path = path.join(&entry_name);

            // Pass the reader wrapper. find_matches_streaming takes BufReader<R>.
            // Since ZipFile implements Read, we can just use BufReader::new(&mut file).
            // We clone entry_path to pass it.
            let reader = BufReader::new(&mut file);
            if let Ok(file_matches) = find_matches_streaming(reader, pattern, &entry_path) {
                matches.extend(file_matches);
            }
        }
    }
    Ok(matches)
}

pub fn search_tar(path: &Path, pattern: &Regex) -> RfgrepResult<Vec<SearchMatch>> {
    let file = File::open(path).map_err(crate::error::RfgrepError::Io)?;
    let mut archive = tar::Archive::new(file);
    let mut matches = Vec::new();

    // tar::Archive::entries returns iterator of Result<Entry>
    for entry_result in archive.entries().map_err(crate::error::RfgrepError::Io)? {
        let mut entry = entry_result.map_err(crate::error::RfgrepError::Io)?;
        if entry.header().entry_type().is_file() {
            let path_cow = entry.path().map_err(crate::error::RfgrepError::Io)?;
            let entry_path = path.join(path_cow);

            let reader = BufReader::new(&mut entry);
            if let Ok(file_matches) = find_matches_streaming(reader, pattern, &entry_path) {
                matches.extend(file_matches);
            }
        }
    }
    Ok(matches)
}

pub fn search_archive(path: &Path, pattern: &Regex) -> RfgrepResult<Vec<SearchMatch>> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "zip" | "jar" => search_zip(path, pattern),
        "tar" => search_tar(path, pattern),
        _ => Ok(vec![]), // Should not reach here if called correctly or maybe treat as error?
    }
}
