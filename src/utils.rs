use std::{path::PathBuf, process::Stdio, str::FromStr};

use ashpd::desktop::file_chooser::OpenFileRequest;
use dirs::data_dir;
use time::OffsetDateTime;

pub fn get_app_name(desktop_file: &str) -> Result<Option<String>, std::io::Error> {
    let user_applications_path = format!("{}/applications/", dirs::data_dir().unwrap().display());
    let search_paths = vec![
        "/usr/share/applications/",
        &user_applications_path,
        "/var/lib/flatpak/exports/share/applications/",
        "",
    ];
    let desktop_file_path = search_paths.iter().fold(None, |acc, search_path| {
        let path = format!("{}{}.desktop", search_path, desktop_file);
        if std::fs::exists(&path).unwrap() {
            Some(path)
        } else {
            acc
        }
    });

    Ok(if let Some(desktop_file_path) = desktop_file_path {
        let parsed = freedesktop_entry_parser::parse_entry(desktop_file_path)?;
        Some(
            parsed
                .section("Desktop Entry")
                .ok_or(std::io::ErrorKind::InvalidData)?
                .attr("Name")[0]
                .clone(),
        )
    } else {
        None
    })
}

pub fn get_script_path() -> Option<PathBuf> {
    let mut script_path = data_dir().unwrap();
    script_path.push("kwin_script.js");
    if is_flatpak() {
        std::fs::copy("/app/share/kwin_script.js", &script_path).unwrap();
    }

    let local_path = std::env::current_dir().unwrap().join("dist/kwin_script.js");
    let search_paths = vec![
        "/usr/share/trayplay/kwin_script.js",
        &script_path.as_os_str().to_str().unwrap(),
        local_path.to_str().unwrap(),
    ];

    search_paths.iter().fold(None, |acc, search_path| {
        if std::fs::exists(search_path).unwrap() {
            Some(PathBuf::from_str(search_path).unwrap())
        } else {
            acc
        }
    })
}

pub async fn ask_path(
    directory: bool,
    initial: &PathBuf,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let request = OpenFileRequest::default()
        .directory(directory)
        .current_folder(initial)?
        .send()
        .await
        .and_then(|r| r.response());

    match request {
        Ok(directory) => {
            let directory = directory.uris()[0].to_file_path().unwrap();
            Ok(Some(directory))
        }
        Err(err) => match err {
            ashpd::Error::Response(response_error) => match response_error {
                ashpd::desktop::ResponseError::Cancelled => Ok(None),
                err => Err(err.into()),
            },
            err => Err(err.into()),
        },
    }
}

pub fn process_pattern(pattern: &str, app_name: &str) -> String {
    let local_time = OffsetDateTime::now_local().unwrap();

    pattern
        .replace("%app%", app_name)
        .replace("%year%", &pad_date_component(local_time.year().to_string()))
        .replace(
            "%month%",
            &pad_date_component((local_time.month() as usize).to_string()),
        )
        .replace("%day%", &pad_date_component(local_time.day().to_string()))
        .replace("%hour%", &pad_date_component(local_time.hour().to_string()))
        .replace(
            "%minute%",
            &pad_date_component(local_time.minute().to_string()),
        )
        .replace(
            "%second%",
            &pad_date_component(local_time.second().to_string()),
        )
}

pub fn get_command_output(command: &str, args: &[&str]) -> Result<String, std::io::Error> {
    Ok(String::from_utf8_lossy(
        &std::process::Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap()
            .stdout,
    )
    .into())
}

fn pad_date_component(input: String) -> String {
    format!("{:0>2}", input)
}

pub fn is_flatpak() -> bool {
    std::env::var("container").is_ok()
}

pub fn is_kde() -> bool {
    std::env::var("KDE").is_ok()
}
