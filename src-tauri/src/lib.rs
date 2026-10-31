//! Haunt's desktop app. The window calls Rust through typed commands (api.rs; their TypeScript is app/bindings.ts);
//! Rust keeps the tabs and the settings, reads git (git/), watches the repositories, opens the editor, and runs the
//! TypeScript scan in a sidecar per shown repository, on Node, Bun or Deno (sidecar.rs). In a release build nothing
//! listens on a port but askpass.rs, on the loopback address with a secret, for git's questions while it fetches,
//! pulls or pushes.
//! While Haunt is worked on, the same commands also answer on 127.0.0.1 for the dev server's page (devbridge.rs).

// First: the said! macro is used by the modules after it.
#[macro_use]
mod said;
mod alerts;
mod api;
mod appimage;
mod askpass;
mod code;
mod desktop;
#[cfg(debug_assertions)]
mod devbridge;
mod editor;
mod events;
mod export;
mod first_frame;
mod folders;
mod git;
mod hosts;
mod icons;
mod paths;
mod process;
mod repos;
mod settings;
mod sidecar;
mod startup;
mod testrun;
mod theme_file;
mod tryit;
mod watch;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{DragDropEvent, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_window_state::StateFlags;

use api::Api;
use events::Events;
use repos::{Context, Repos};

/// The commands, for Tauri and for the window's generated bindings.
fn bindings() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(api::collect())
        // One shape per type, and a failed command throws its message, as every caller in the window expects.
        .disable_serde_phases()
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        // Times are whole seconds since 1970, far inside what a JavaScript number holds exactly.
        .dangerously_cast_bigints_to_number()
}

/// Haunt's program started by git to ask something: its exit code; None when started normally.
pub fn askpass_main() -> Option<i32> {
    askpass::main()
}

pub fn run() {
    let commands = bindings();
    tauri::Builder::default()
        // First: a second `haunt <folder>` (or a second start from the menu) opens its folder in the window already
        // running, and brings it to the front, instead of starting another Haunt.
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            let Some(folder) = args.iter().skip(1).find(|arg| !arg.starts_with('-')) else { return };
            let path = std::path::Path::new(&cwd).join(folder).to_string_lossy().into_owned();
            let api = app.state::<Arc<Api>>().inner().clone();
            std::thread::spawn(move || {
                if let Err(problem) = api.repos.open(&path) {
                    api.repos.notice(&problem);
                }
            });
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        // Remember where the window was and how big, not whether it had a frame: the frame is Haunt's choice (below),
        // and a state saved by a build with the system's frame would bring it back.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .build(),
        )
        .invoke_handler(commands.invoke_handler())
        // A folder dropped on the window opens in its tab (the first folder, when several are dropped).
        .on_window_event(|window, event| {
            if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
                let api = window.state::<Arc<Api>>().inner().clone();
                let paths = paths.clone();
                std::thread::spawn(move || {
                    let Some(folder) = paths.iter().find(|path| path.is_dir()) else {
                        return api.repos.notice(&said!(
                            "lib.dropFolder",
                            "Drop a folder to open it: a project or a git repository."
                        ));
                    };
                    if let Err(problem) = api.repos.open(&folder.to_string_lossy()) {
                        api.repos.notice(&problem);
                    }
                });
            }
        })
        .setup(|app| {
            let config = paths::plain(app.path().app_config_dir()?);
            let logs = paths::plain(app.path().app_log_dir()?);
            std::fs::create_dir_all(&config)?;
            std::fs::create_dir_all(&logs)?;
            let settings_file = settings::file(config.clone());

            alerts::start(app.handle().clone());
            let mut problems = Vec::new();
            let runtime = startup::find_runtime();
            if let Err(problem) = &runtime {
                problems.push(problem.clone());
            }
            if let Err(problem) = startup::check_git() {
                problems.push(problem);
            }
            let script = sidecar_script(app.handle())?;
            let events = Arc::new(Events::default());
            let handle = app.handle().clone();
            events.attach(Box::new(move |news| {
                let _ = handle.emit("haunt", news);
            }));
            // git's questions during a fetch, pull or push, asked in the window.
            askpass::start(events.clone())?;
            // The desktop's light or dark, accent and window buttons, followed (Linux).
            desktop::start(events.clone());
            // theme.json, which a wallpaper tool may write; read before the window is made (its opacity).
            if let Ok(config_root) = app.path().config_dir() {
                theme_file::start(&paths::plain(config_root), events.clone());
            }
            let log = (!cfg!(debug_assertions)).then(|| logs.join("scanner.log"));
            let cache = app.path().app_cache_dir().ok().map(paths::plain).map(|dir| dir.join("calls"));
            if let Some(cache) = &cache {
                let _ = std::fs::create_dir_all(cache);
            }
            let logs_folder = (!cfg!(debug_assertions)).then(|| logs.clone());
            let repos = Repos::new(
                Context { runtime, script, settings: settings_file, log, cache, logs: logs_folder },
                events.clone(),
            );
            let api = Arc::new(Api { repos: repos.clone(), problems });
            app.manage(api.clone());
            // Fetches the repository shown now and then, when Settings asks for it.
            api::fetch_in_background(api.clone());
            let system_title_bar = api.repos.system_title_bar();
            #[cfg(debug_assertions)]
            devbridge::start(api, events);
            #[cfg(not(debug_assertions))]
            let _ = (api, events);

            let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(PathBuf::from("index.html")))
                .title("Haunt")
                .inner_size(1440.0, 920.0)
                .min_inner_size(900.0, 600.0);
            // The page's own colour from the first frame, not white while the page loads.
            let builder = match first_frame::color(&repos.info().appearance) {
                Some(color) => builder.background_color(color),
                None => builder,
            };
            // The tabs are the title bar: on macOS over the system's traffic lights, elsewhere with Haunt's own window
            // buttons and no system frame (the shadow keeps the edges for resizing).
            #[cfg(target_os = "macos")]
            let builder = builder.title_bar_style(tauri::TitleBarStyle::Overlay).hidden_title(true);
            #[cfg(not(target_os = "macos"))]
            let builder = builder.decorations(false).shadow(true);
            // A theme file's opacity below 1 lets the compositor's blur through the window's background (Linux): the
            // window has to be made transparent for that, so it is read here, once.
            #[cfg(target_os = "linux")]
            let builder = builder.transparent(theme_file::transparent_window());
            let window = builder.build()?;
            // Again once built, after the window-state plugin has restored what it keeps; the system's frame when
            // Settings asks for it.
            #[cfg(not(target_os = "macos"))]
            window.set_decorations(system_title_bar)?;
            #[cfg(target_os = "macos")]
            let _ = (window, system_title_bar);

            // The folder named on the command line (`haunt .`), or else the repository shown last time, starts
            // scanning while the window loads.
            let asked = std::env::args().skip(1).find(|arg| !arg.starts_with('-'));
            std::thread::spawn(move || match asked.map(|folder| repos.open(&folder)) {
                Some(Ok(_)) => {}
                Some(Err(problem)) => {
                    repos.start();
                    repos.notice(&problem);
                }
                None => repos.start(),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the app could not start");
}

/// `sidecar/main.ts`: next to the app's resources in a release build, in the source folder while Haunt is worked on.
fn sidecar_script(app: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Ok(paths::plain(std::fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/../sidecar/main.ts"))?));
    }
    Ok(paths::plain(app.path().resource_dir()?).join("sidecar").join("main.ts"))
}
