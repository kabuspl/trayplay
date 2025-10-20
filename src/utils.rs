use std::{path::PathBuf, str::FromStr};

use ashpd::desktop::file_chooser::OpenFileRequest;
use time::{OffsetDateTime, PrimitiveDateTime, UtcDateTime};

use crate::kdialog::{self, InfoBox, InputBox};

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
        parsed
            .section("Desktop Entry")
            .attr("Name")
            .map(|v| v.to_string())
    } else {
        None
    })
}

pub fn get_script_path() -> Option<PathBuf> {
    let local_path = std::env::current_dir().unwrap().join("dist/kwin_script.js");
    let search_paths = vec![
        "/usr/share/trayplay/kwin_script.js",
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

pub fn ask_custom_number(
    title: &str,
    label: &str,
    initial: impl Into<i64>,
) -> Result<Option<i64>, Box<dyn std::error::Error>> {
    let initial = initial.into();

    let result = InputBox::new(label, kdialog::InputBoxType::Text)
        .initial(initial.to_string())
        .title(title)
        .show()?;

    if let Some(result) = result {
        let number = result.replace("\n", "").parse::<i64>();
        if let Ok(number) = number {
            Ok(Some(number))
        } else {
            InfoBox::warning("You need to input an integer.")
                .title("Wrong input")
                .show()?;

            ask_custom_number(title, label, initial)
        }
    } else {
        Ok(None)
    }
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
        .replace("%year%", &local_time.year().to_string())
        .replace("%month%", &(local_time.month() as usize).to_string())
        .replace("%day%", &local_time.day().to_string())
        .replace("%hour%", &local_time.hour().to_string())
        .replace("%minute%", &local_time.minute().to_string())
        .replace("%second%", &local_time.second().to_string())
}
