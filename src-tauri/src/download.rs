//! Download Instagram post media through the instaloader engine.

use crate::parse::DownloadTarget;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ENGINE: &str = "instaloader";
const MEDIA_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "bmp", "heic", "mp4", "m4v", "mov", "webm", "mkv",
];

#[derive(Debug, Clone, Serialize)]
pub struct DownloadResult {
    pub shortcode: String,
    pub engine: String,
    pub command: Vec<String>,
    pub destination: String,
    pub success: bool,
    pub media_files: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
}

const BUNDLED_NAMES: &[&str] = &[
    "instaloader.cmd",
    "instaloader.exe",
    "instaloader.bat",
    "instaloader",
];

fn bundled_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("INS_RESOURCE_DIR") {
        let dir = PathBuf::from(dir);
        roots.push(dir.join("resources"));
        roots.push(dir.clone());
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("../Resources/resources"));
            roots.push(dir.join("../Resources"));
            roots.push(dir.join("resources"));
            roots.push(dir.to_path_buf());
            roots.push(dir.join("../lib/ins-downloader/resources"));
            roots.push(dir.join("../lib/com.ins.downloader/resources"));
        }
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"));
    roots
}

fn bundled_instaloader() -> Option<PathBuf> {
    for root in bundled_roots() {
        for name in BUNDLED_NAMES {
            let path = root.join("instaloader-runtime").join(name);
            if path.is_file() {
                return Some(path.canonicalize().unwrap_or(path));
            }
        }
    }
    None
}

fn instaloader_invocation(bin: &Path) -> (Command, Vec<String>) {
    if let Some(dir) = bin.parent() {
        let python_candidates = [
            dir.join("bin").join("python3"),
            dir.join("python.exe"),
            dir.join("bin").join("python.exe"),
        ];
        for py in python_candidates {
            if py.is_file() {
                let mut cmd = Command::new(&py);
                cmd.arg("-m").arg("instaloader");
                return (
                    cmd,
                    vec![
                        py.display().to_string(),
                        "-m".into(),
                        "instaloader".into(),
                    ],
                );
            }
        }
    }
    #[cfg(windows)]
    {
        let is_batch = bin
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
            .unwrap_or(false);
        if is_batch {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C");
            cmd.arg(bin);
            return (cmd, vec![bin.display().to_string()]);
        }
    }
    (
        Command::new(bin),
        vec![bin.display().to_string()],
    )
}

fn prepend_path(dir: Option<&Path>) -> std::ffi::OsString {
    let mut paths = Vec::new();
    if let Some(dir) = dir {
        paths.push(dir.to_path_buf());
    }
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(paths).unwrap_or_else(|_| std::env::var_os("PATH").unwrap_or_default())
}

/// Locate the instaloader executable (bundled copy first, then system).
pub fn find_instaloader() -> Result<PathBuf, String> {
    if let Ok(explicit) = std::env::var("INSTALOADER") {
        let path = PathBuf::from(&explicit);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("INSTALOADER is set but not a file: {explicit}"));
    }

    if let Some(path) = bundled_instaloader() {
        return Ok(path);
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let home = PathBuf::from(home);
        candidates.push(home.join(".local/bin/instaloader"));
        candidates.push(home.join(".local/share/uv/tools/instaloader/bin/instaloader"));
    }
    candidates.push(PathBuf::from("/opt/homebrew/bin/instaloader"));
    candidates.push(PathBuf::from("/usr/local/bin/instaloader"));

    for path in &candidates {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    if let Some(path) = which_instaloader() {
        return Ok(path);
    }

    Err("未找到 instaloader。安装包应已内置；开发时请运行 scripts/vendor-instaloader.sh".into())
}

fn which_instaloader() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let output = Command::new("where").arg("instaloader").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let line = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if line.is_empty() {
            None
        } else {
            Some(PathBuf::from(line))
        }
    }
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("/usr/bin/which");
        cmd.arg("instaloader");
        if let Ok(home) = std::env::var("HOME") {
            let extra = format!("{home}/.local/bin:/opt/homebrew/bin:/usr/local/bin");
            let path = std::env::var("PATH").unwrap_or_default();
            cmd.env("PATH", format!("{extra}:{path}"));
        }
        let output = cmd.output().ok()?;
        if !output.status.success() {
            return None;
        }
        let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if line.is_empty() {
            None
        } else {
            Some(PathBuf::from(line))
        }
    }
}

fn is_media(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| MEDIA_EXTS.iter().any(|want| e.eq_ignore_ascii_case(want)))
        .unwrap_or(false)
}

fn collect_target_media(destination: &Path, target: &DownloadTarget) -> Vec<String> {
    let post_dir = destination.join(target.folder_name());
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&post_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_media(&path) {
                files.push(path.display().to_string());
            }
        }
    }
    files.sort();
    files
}

/// Collect image/video files under dest (dest itself and one subdirectory level).
pub fn collect_media_files(root: &Path) -> HashSet<PathBuf> {
    let mut files = HashSet::new();
    let Ok(entries) = fs::read_dir(root) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(sub) = fs::read_dir(&path) {
                for child in sub.flatten() {
                    let child_path = child.path();
                    if child_path.is_file() && is_media(&child_path) {
                        files.insert(child_path);
                    }
                }
            }
        } else if path.is_file() && is_media(&path) {
            files.insert(path);
        }
    }
    files
}

fn instaloader_args(target: &DownloadTarget, destination: &Path) -> Vec<String> {
    let dest = destination.display().to_string();
    let dirname = if target.is_post() {
        format!("{dest}/{{shortcode}}")
    } else {
        format!("{dest}/{{target}}")
    };
    vec![
        "--dirname-pattern".into(),
        dirname,
        "--filename-pattern".into(),
        "{shortcode}".into(),
        "--no-captions".into(),
        "--no-metadata-json".into(),
        "--no-video-thumbnails".into(),
        "--max-connection-attempts".into(),
        "2".into(),
        "--request-timeout".into(),
        "60".into(),
        "--abort-on".into(),
        "401,403,429".into(),
        "--".into(),
        target.instaloader_target(),
    ]
}

/// Download one post’s images and/or videos into `destination` using instaloader.
pub fn download_post(shortcode: &str, destination: &Path) -> DownloadResult {
    download_target(&DownloadTarget::Post(shortcode.to_string()), destination)
}

/// Download one instaloader target into `destination`.
pub fn download_target(target: &DownloadTarget, destination: &Path) -> DownloadResult {
    let dest_str = destination.display().to_string();
    let mut result = DownloadResult {
        shortcode: target.display_label(),
        engine: ENGINE.to_string(),
        command: Vec::new(),
        destination: dest_str,
        success: false,
        media_files: Vec::new(),
        stdout: String::new(),
        stderr: String::new(),
        error: None,
    };

    if target.display_label().is_empty() {
        result.error = Some("empty target".into());
        return result;
    }

    if let Err(err) = fs::create_dir_all(destination) {
        result.error = Some(format!("无法创建目标目录: {err}"));
        return result;
    }

    let bin = match find_instaloader() {
        Ok(p) => p,
        Err(err) => {
            result.error = Some(err);
            return result;
        }
    };

    let args = instaloader_args(target, destination);
    let (mut cmd, mut recorded) = instaloader_invocation(&bin);
    recorded.extend(args.iter().cloned());
    result.command = recorded;

    let before = collect_media_files(destination);

    cmd.args(&args);
    cmd.env("PATH", prepend_path(bin.parent()));

    let output = match cmd.output() {
        Ok(o) => o,
        Err(err) => {
            result.error = Some(format!("启动 instaloader 失败: {err}"));
            return result;
        }
    };

    result.stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    result.stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let after = collect_media_files(destination);
    let mut new_files: Vec<String> = after
        .difference(&before)
        .map(|p| p.display().to_string())
        .collect();
    new_files.sort();
    if new_files.is_empty() {
        // Re-download of the same post: instaloader may skip existing files.
        result.media_files = collect_target_media(destination, target);
    } else {
        result.media_files = new_files;
    }

    let non_empty = result.media_files.iter().any(|p| {
        fs::metadata(p)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    });

    if output.status.success() && non_empty {
        result.success = true;
    } else if !result.media_files.is_empty() && non_empty {
        result.success = true;
    } else {
        let combined = format!("{}\n{}", result.stdout, result.stderr);
        result.error = Some(
            if combined.trim().is_empty() {
                format!(
                    "instaloader 退出码 {:?} 且未写入媒体文件",
                    output.status.code()
                )
            } else {
                combined.trim().to_string()
            },
        );
    }

    result
}

/// Batch-download each target independently into the same destination.
pub fn download_posts(targets: &[DownloadTarget], destination: &Path) -> Vec<DownloadResult> {
    targets
        .iter()
        .map(|target| download_target(target, destination))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    static INSTALOADER_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn instaloader_args_match_documented_targets() {
        let dest = Path::new("/tmp/ins-dl");
        let post = instaloader_args(&DownloadTarget::Post("DdivRY4CFX6".into()), dest);
        assert!(post.iter().any(|a| a == "-DdivRY4CFX6"));
        assert!(post.iter().any(|a| a.contains("{shortcode}")));

        let profile = instaloader_args(&DownloadTarget::Profile("natgeo".into()), dest);
        assert!(profile.iter().any(|a| a == "natgeo"));
        assert!(profile.iter().any(|a| a.ends_with("{target}")));

        let hashtag = instaloader_args(&DownloadTarget::Hashtag("kitten".into()), dest);
        assert!(hashtag.iter().any(|a| a == "#kitten"));

        let location = instaloader_args(&DownloadTarget::Location("362629379".into()), dest);
        assert!(location.iter().any(|a| a == "%362629379"));
    }

    #[test]
    fn find_instaloader_resolves_an_executable_named_instaloader() {
        let bin = find_instaloader().expect("instaloader must be bundled or on PATH");
        let name = bin.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert!(
            name.contains("instaloader"),
            "resolved engine is not instaloader: {}",
            bin.display()
        );
        let output = instaloader_invocation(&bin)
            .0
            .arg("--version")
            .output()
            .expect("run instaloader");
        assert!(
            output.status.success(),
            "instaloader --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn unique_dest() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "ins-dl-{}-{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn assert_instaloader_engine(result: &DownloadResult, shortcode: &str) {
        assert_eq!(result.engine, "instaloader");
        assert!(
            !result.command.is_empty(),
            "instaloader was not invoked; error={:?}",
            result.error
        );
        let bin = &result.command[0];
        assert!(
            result.command.iter().any(|a| a.contains("instaloader")),
            "engine binary is not instaloader: {bin}"
        );
        assert!(
            result.command.iter().any(|a| a == "--"),
            "missing instaloader shortcode separator: {:?}",
            result.command
        );
        let target = format!("-{shortcode}");
        assert!(
            result.command.iter().any(|a| a == &target),
            "missing shortcode target {target}: {:?}",
            result.command
        );
        assert!(
            result
                .command
                .iter()
                .any(|a| a == "--dirname-pattern" || a.contains("{shortcode}")),
            "destination pattern not passed to instaloader: {:?}",
            result.command
        );
    }

    fn assert_has_nonempty_media(result: &DownloadResult) {
        eprintln!("engine={} command={:?}", result.engine, result.command);
        assert!(
            result.success,
            "download failed for {}: {}\nstdout:\n{}\nstderr:\n{}",
            result.shortcode,
            result.error.as_deref().unwrap_or("(no error text)"),
            result.stdout,
            result.stderr
        );
        assert!(
            !result.media_files.is_empty(),
            "no media files for shortcode {}",
            result.shortcode
        );
        eprintln!(
            "instaloader wrote {} media file(s) for {}:",
            result.media_files.len(),
            result.shortcode
        );
        for path in &result.media_files {
            let meta = fs::metadata(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(meta.len() > 0, "media file is empty: {path}");
            assert!(
                is_media(Path::new(path)),
                "not an image/video file: {path}"
            );
            eprintln!("  {path} ({} bytes)", meta.len());
        }
    }

    #[test]
    fn download_one_post_uses_instaloader() {
        let _guard = INSTALOADER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dest = unique_dest();
        let shortcode = "DdivRY4CFX6";
        let result = download_post(shortcode, &dest);
        assert_instaloader_engine(&result, shortcode);
        assert_has_nonempty_media(&result);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn download_batch_uses_instaloader_for_each_shortcode() {
        let _guard = INSTALOADER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dest = unique_dest();
        // Proven-good shortcode used twice so every requested post has media files.
        let codes = vec![
            DownloadTarget::Post("DdivRY4CFX6".into()),
            DownloadTarget::Post("DdivRY4CFX6".into()),
        ];
        let results = download_posts(&codes, &dest);
        assert_eq!(results.len(), 2);
        for (result, code) in results.iter().zip(codes.iter()) {
            assert_instaloader_engine(result, &code.display_label());
            assert_has_nonempty_media(result);
        }
        let on_disk = collect_media_files(&dest);
        assert!(
            !on_disk.is_empty(),
            "destination has no media files after batch download"
        );
        for path in &on_disk {
            let meta = fs::metadata(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(meta.len() > 0, "empty media file on disk: {}", path.display());
        }
        let _ = fs::remove_dir_all(&dest);
    }
}
