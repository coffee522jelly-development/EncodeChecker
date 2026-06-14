use std::fs::File;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;
use serde::Serialize;

/// 各ファイルの検査結果を格納する構造体
#[derive(Serialize)]
pub struct FileResult {
    pub path: String,     // ルートフォルダからの相対パス
    pub status: String,   // 判定結果 (OK, NG, UNKNOWN, BINARY)
    pub encoding: String, // 判定された文字コード
    pub newline: String,  // 判定された改行コード
    pub size: u64,        // ファイルサイズ (bytes)
}

/// 指定されたフォルダ内のファイルを再帰的に走査し、判定を行うメインロジック
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
        return Err(format!("指定されたフォルダが存在しません: {}", folder));
    }

    // WalkDirを使用してディレクトリを再帰的に走査
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // ディレクトリ自体はスキップ
        if path.is_dir() {
            continue;
        }

        // 除外フォルダの判定 (パスの構成要素に除外対象が含まれているか確認)
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

        // 除外拡張子の判定
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if exclude_exts.contains(&format!(".{}", ext)) {
                continue;
            }
        }

        // 個別ファイルの判定処理を実行
        let file_result = process_file(path, root, &expected_encoding, &expected_newline);
        results.push(file_result);
    }

    Ok(results)
}

/// 単一のファイルを読み込み、文字コードと改行コードを判定する
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
    // 判定のために最大1MBまで読み込む (巨大なファイル対策)
    let mut chunk = vec![0; 1024 * 1024];
    let n = file.read(&mut chunk).unwrap_or(0);
    buffer.extend_from_slice(&chunk[..n]);

    // バイナリファイルかどうかのチェック (NULLバイトによる簡易判定)
    if is_binary(&buffer) {
        return FileResult {
            path: relative_path,
            status: "BINARY".to_string(),
            encoding: "Binary".to_string(),
            newline: "Binary".to_string(),
            size,
        };
    }

    // 文字コードと改行コードの検出
    let detected_enc = detect_encoding(&buffer);
    let detected_nl = detect_newline(&buffer);

    // 期待値と比較してステータスを決定
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

/// バイナリファイルかどうかを判定する (簡易的にNULLバイトの有無で判断)
pub fn is_binary(buffer: &[u8]) -> bool {
    // 最初の8KB程度を確認
    buffer.iter().take(8192).any(|&b| b == 0)
}

/// 文字コードを推定する (chardetngを使用)
pub fn detect_encoding(buffer: &[u8]) -> String {
    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(buffer, true);
    let encoding = detector.guess(None, chardetng::Utf8Detection::Allow);
    encoding.name().to_string()
}

/// 改行コードを判定する
pub fn detect_newline(buffer: &[u8]) -> String {
    let has_cr = buffer.contains(&b'\r');
    let has_lf = buffer.contains(&b'\n');

    if has_cr && has_lf {
        // \r\n (CRLF) が含まれているかスキャン
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
    // アプリケーションの実行（構造維持のためのダミー）
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
