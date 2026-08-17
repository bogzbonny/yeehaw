/// Example demonstrating TerminalPane2, the passively-drawn terminal pane,
/// running the `sl` command which produces dynamic terminal output.
use portable_pty::CommandBuilder;
use yeehaw::*;

/// Returns a command builder for `sl` if it is on the PATH, otherwise a
/// command builder for the default shell.
fn sl_command() -> CommandBuilder {
    let cwd = std::env::current_dir().unwrap_or_default();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join("sl").is_file() {
                let mut cmd = CommandBuilder::new("sl");
                cmd.cwd(cwd);
                return cmd;
            }
        }
    }
    eprintln!("sl not found on PATH; falling back to the default shell");
    let mut cmd = CommandBuilder::new_default_prog();
    cmd.cwd(cwd);
    cmd
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let pane = TerminalPane2::new_with_builder(&ctx, sl_command())?;
    tui.run(Box::new(pane)).await
}
