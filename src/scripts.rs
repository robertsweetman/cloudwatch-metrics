use tokio::process::Command;
use std::path::Path;
use cloudwatch_metrics::Script;

pub async fn run_script_and_get_status(script_paths: &Script) -> Result<i32, std::io::Error> {
    let output = Command::new("sudo")
        .arg("-u")
        .arg(&script_paths.user)
        .arg("bash")
        .arg(&script_paths.path)
        .output()
        .await?;

    Ok(output.status.code().unwrap_or(-1))
}

pub fn get_filename_from_path(path: &str) -> Option<&str> {
    let path = Path::new(path);
    let script_name = path.file_stem()?;
    script_name.to_str()
}