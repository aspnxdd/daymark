#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod model;
mod storage;
mod ui;

use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use freya::prelude::*;

fn main() -> Result<()> {
    let Some(directory) = data_directory(std::env::args_os().skip(1).collect())? else {
        return Ok(());
    };
    let store = storage::Store::open(&directory).with_context(|| {
        format!(
            "Could not open {}. Your existing data has not been replaced.",
            directory.display()
        )
    })?;
    let unsaved = Arc::new(AtomicBool::new(false));
    let journal = model::Journal::new(store, unsaved.clone())?;
    launch(
        LaunchConfig::new()
            .with_font(
                "DM Sans",
                include_bytes!("../assets/fonts/DMSans.ttf").as_slice(),
            )
            .with_font(
                "DM Serif Display",
                include_bytes!("../assets/fonts/DMSerifDisplay-Regular.ttf").as_slice(),
            )
            .with_default_font("DM Sans")
            .with_window(
                WindowConfig::new_app(ui::DaymarkApp {
                    initial: RefCell::new(Some(journal)),
                })
                .with_title("Daymark · A little space for your days")
                .with_app_id("dev.daymark.journal")
                .with_size(1360., 920.)
                .with_min_size(1060., 760.)
                .with_background(ui::theme::COLORS.background)
                .with_on_close(move |_, _| {
                    if unsaved.load(Ordering::SeqCst) {
                        CloseDecision::KeepOpen
                    } else {
                        CloseDecision::Close
                    }
                }),
            ),
    );
    Ok(())
}

fn data_directory(args: Vec<std::ffi::OsString>) -> Result<Option<PathBuf>> {
    match args.as_slice() {
        [flag] if flag == "--help" || flag == "-h" => {
            println!(
                "Daymark: a local-only journal\n\nUsage: daymark [--data-dir PATH]\n\nUse a dedicated data directory. No network, accounts, or calendar.\nThe local SQLite database is NOT encrypted. See docs/PRIVACY.md."
            );
            Ok(None)
        }
        [flag, path] if flag == "--data-dir" && !path.is_empty() => Ok(Some(PathBuf::from(path))),
        [] => Ok(Some(
            ProjectDirs::from("dev", "daymark", "Daymark")
                .context(
                    "Could not locate your local application data directory. Use --data-dir PATH.",
                )?
                .data_local_dir()
                .to_path_buf(),
        )),
        _ => bail!("Unknown arguments. Use daymark --help."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_data_directory_uses_daymark_identity() {
        let expected = ProjectDirs::from("dev", "daymark", "Daymark").unwrap();
        assert_eq!(
            data_directory(Vec::new()).unwrap(),
            Some(expected.data_local_dir().to_path_buf())
        );
    }

    #[test]
    fn cli_accepts_only_explicit_data_directory_or_help() {
        assert_eq!(
            data_directory(vec!["--data-dir".into(), "/tmp/daymark-test".into()]).unwrap(),
            Some(PathBuf::from("/tmp/daymark-test"))
        );
        assert!(data_directory(vec!["--help".into()]).unwrap().is_none());
        assert!(data_directory(vec!["--data-dir".into()]).is_err());
        assert!(data_directory(vec!["--unknown".into()]).is_err());
        assert!(data_directory(vec!["--data-dir".into(), "".into()]).is_err());
    }
}
