use std::fs::File;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileResult {
    pub path: String,
    pub status: String,
    pub encoding: String,
    pub newline: String,
    pub size: u64,
}

pub async fn check_folder_logic(
    folder: String,
    expected_encoding: String,
    expected_newline: String,
    exclude_dirs: Vec<String>,
    exclude_exts: Vec<String>,
) -> Result<Vec<FileResult>, String> {
    let mut results = Vec::new();
    let root = Path::new(&folder);

    if !root.exists() {
        return Err(format!("Folder does not exist: {}", folder));
    }

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        // Check if path should be excluded
        let is_excluded_dir = path.components().any(|c| {
            if let Some(s) = c.as_os_str().to_str() {
                exclude_dirs.contains(&s.to_string())
            } else {
                false
            }
        });
        if is_excluded_dir {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if exclude_exts.contains(&format!(".{}", ext)) {
                continue;
            }
        }

        let file_result = process_file(path, root, &expected_encoding, &expected_newline);
        results.push(file_result);
    }

    Ok(results)
}

pub fn process_file(path: &Path, root: &Path, expected_enc: &str, expected_nl: &str) -> FileResult {
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned();
    let metadata = std::fs::metadata(path);
    let size = metadata.map(|m| m.len()).unwrap_or(0);

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return FileResult {
            path: relative_path,
            status: "UNKNOWN".to_string(),
            encoding: "Error".to_string(),
            newline: "Error".to_string(),
            size,
        },
    };

    let mut buffer = Vec::new();
    // Read up to 1MB for detection
    let mut chunk = vec![0; 1024 * 1024];
    let n = file.read(&mut chunk).unwrap_or(0);
    buffer.extend_from_slice(&chunk[..n]);

    if is_binary(&buffer) {
        return FileResult {
            path: relative_path,
            status: "BINARY".to_string(),
            encoding: "Binary".to_string(),
            newline: "Binary".to_string(),
            size,
        };
    }

    let detected_enc = detect_encoding(&buffer);
    let detected_nl = detect_newline(&buffer);

    let status = if (detected_enc == expected_enc || (expected_enc == "UTF-8" && detected_enc == "UTF-8")) && detected_nl == expected_nl {
        "OK"
    } else {
        "NG"
    };

    FileResult {
        path: relative_path,
        status: status.to_string(),
        encoding: detected_enc,
        newline: detected_nl,
        size,
    }
}

pub fn is_binary(buffer: &[u8]) -> bool {
    // Simple heuristic: check for null bytes in the first chunk
    buffer.iter().take(8192).any(|&b| b == 0)
}

pub fn detect_encoding(buffer: &[u8]) -> String {
    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(buffer, true);
    let encoding = detector.guess(None, chardetng::Utf8Detection::Allow);
    encoding.name().to_string()
}

pub fn detect_newline(buffer: &[u8]) -> String {
    let has_cr = buffer.contains(&b'\r');
    let has_lf = buffer.contains(&b'\n');

    if has_cr && has_lf {
        // Check if \r is followed by \n
        let mut found_crlf = false;
        for i in 0..buffer.len().saturating_sub(1) {
            if buffer[i] == b'\r' && buffer[i+1] == b'\n' {
                found_crlf = true;
                break;
            }
        }
        if found_crlf {
            "CRLF".to_string()
        } else {
            "Mixed/Other".to_string()
        }
    } else if has_lf {
        "LF".to_string()
    } else if has_cr {
        "CR".to_string()
    } else {
        "None".to_string()
    }
}

pub fn run_app() {
    // This is not used but kept for structure if needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_newline() {
        assert_eq!(detect_newline(b"hello\nworld"), "LF");
        assert_eq!(detect_newline(b"hello\r\nworld"), "CRLF");
        assert_eq!(detect_newline(b"hello\rworld"), "CR");
        assert_eq!(detect_newline(b"hello world"), "None");
    }

    #[test]
    fn test_is_binary() {
        assert_eq!(is_binary(b"hello world"), false);
        assert_eq!(is_binary(b"hello\0world"), true);
    }
}
