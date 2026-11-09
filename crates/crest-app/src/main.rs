mod chrome;
mod engine;
mod icons;
mod window;

use cef::*;
use std::{
    cell::{Cell, OnceCell, RefCell},
    ffi::OsString,
    fs::DirBuilder,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    process::ExitCode,
    ptr::null_mut,
    rc::Rc as Shared,
};
use window::WindowState;

const RELAUNCH_ADDRESS: &str = "crest-launch-address";

#[derive(Default)]
struct Run {
    launch: OnceCell<Launch>,
    window: RefCell<Option<Shared<WindowState>>>,
    failed: Cell<bool>,
}

impl Run {
    fn fail(&self, message: &str) {
        eprintln!("Crest Browser · {message}");
        self.failed.set(true);
        quit_message_loop();
    }
}

struct Launch {
    profile: PathBuf,
    input: Option<String>,
}

enum Command {
    Help,
    Version,
    Launch(Launch),
}

fn parse_command(args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let mut args = args;
    let mut profile = None;
    let mut input = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => return Ok(Command::Help),
            Some("--version" | "-V") => return Ok(Command::Version),
            Some("--profile") if profile.is_none() => {
                profile = Some(PathBuf::from(
                    args.next().ok_or("--profile requires a directory")?,
                ));
            }
            Some(value) if !value.starts_with('-') && input.is_none() => {
                input = Some(value.to_owned())
            }
            _ => {
                return Err(format!(
                    "Unexpected argument {}. See --help.",
                    arg.to_string_lossy()
                ));
            }
        }
    }
    let profile = profile
        .or_else(|| dirs::data_dir().map(|path| path.join("crest-browser")))
        .ok_or("Could not locate a user data directory. Use --profile DIR.")?;
    Ok(Command::Launch(Launch { profile, input }))
}

wrap_browser_process_handler! {
    struct BrowserProcess { run: Shared<Run> }
    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let Some(launch) = self.run.launch.get() else { return };
            if !engine::register_home() {
                self.run.fail("Could not register the new tab page");
                return;
            }
            let window = match WindowState::new(launch.profile.join("profile.json"), launch.input.as_deref()) {
                Ok(window) => window,
                Err(error) => {
                    self.run.fail(&error);
                    return;
                }
            };
            self.run.window.replace(Some(window.clone()));
            if !window.show() {
                self.run.fail("Could not create the desktop window");
            }
        }

        fn on_already_running_app_relaunch(
            &self,
            command_line: Option<&mut CommandLine>,
            _directory: Option<&CefString>,
        ) -> i32 {
            let window = self.run.window.borrow().clone();
            if let Some(window) = window {
                let input = command_line.and_then(|command_line| {
                    let name = CefString::from(RELAUNCH_ADDRESS);
                    (command_line.has_switch(Some(&name)) != 0)
                        .then(|| CefString::from(&command_line.switch_value(Some(&name))).to_string())
                });
                window.relaunch(input.as_deref());
            }
            1
        }
    }
}

wrap_app! {
    struct BrowserApp { run: Shared<Run> }
    impl App {
        fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
            if let Some(registrar) = registrar {
                let flags = SchemeOptions::STANDARD.get_raw() | SchemeOptions::DISPLAY_ISOLATED.get_raw();
                registrar.add_custom_scheme(Some(&CefString::from("crest")), flags as i32);
            }
        }

        fn on_before_command_line_processing(
            &self,
            process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            if process_type.is_none_or(|value| value.to_string().is_empty())
                && let Some(command_line) = command_line
            {
                command_line.append_switch(Some(&CefString::from("site-per-process")));
                if let Some(input) = self.run.launch.get().and_then(|launch| launch.input.as_deref()) {
                    command_line.append_switch_with_value(
                        Some(&CefString::from(RELAUNCH_ADDRESS)),
                        Some(&CefString::from(input)),
                    );
                }
            }
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(BrowserProcess::new(self.run.clone()))
        }
    }
}

fn run_browser(
    args: &MainArgs,
    app: &mut App,
    run: &Run,
    mut launch: Launch,
) -> Result<(), String> {
    let engine_profile = launch.profile.join("chromium");
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&engine_profile)
        .map_err(|error| format!("Could not create {}: {error}", engine_profile.display()))?;
    launch.profile = launch
        .profile
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let engine_profile = engine_profile
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let engine_profile = engine_profile
        .to_str()
        .ok_or("The profile path must be valid UTF-8")?;
    let settings = Settings {
        command_line_args_disabled: 1,
        root_cache_path: CefString::from(engine_profile),
        cache_path: CefString::from(engine_profile),
        persist_session_cookies: 1,
        ..Default::default()
    };
    let _ = run.launch.set(launch);
    if initialize(Some(args), Some(&settings), Some(app), null_mut()) == 0 {
        let code = get_exit_code();
        return if code == Resultcode::NORMAL_EXIT_PROCESS_NOTIFIED.get_raw() as i32 {
            Ok(())
        } else {
            Err(format!(
                "Chromium initialization did not complete (CEF exit code {code})"
            ))
        };
    }
    if !run.failed.get() {
        run_message_loop();
    }
    run.window.take();
    shutdown();
    if run.failed.get() {
        return Err("Desktop startup did not complete".into());
    }
    Ok(())
}

fn main() -> ExitCode {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let run = Shared::new(Run::default());
    let mut app = BrowserApp::new(run.clone());
    let child_exit = execute_process(Some(args.as_main_args()), Some(&mut app), null_mut());
    if child_exit >= 0 {
        std::process::exit(child_exit);
    }

    let result = match parse_command(std::env::args_os().skip(1)) {
        Ok(Command::Help) => {
            println!(
                "Crest Browser\n\nUsage: crest-browser [--profile DIRECTORY] [ADDRESS]\n       crest-browser --version"
            );
            Ok(())
        }
        Ok(Command::Version) => {
            println!("Crest Browser {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Ok(Command::Launch(launch)) => run_browser(args.as_main_args(), &mut app, &run, launch),
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Crest Browser · {error}");
            ExitCode::FAILURE
        }
    }
}
