#![feature(io_error_more)]

use exec::run_script;

mod exec;
mod line_parser;
mod redirect;
mod term;

#[cfg(unix)]
pub mod term_impl_unix;

struct Cleanup {}
impl Drop for Cleanup {
    fn drop(&mut self) {
        term::on_exit();
    }
}

fn print_usage_and_exit(code: i32) -> ! {
    eprintln!("(rush) usage:");
    eprintln!("    -h: print this message");
    eprintln!("    -i: interactive mode: read commands from stdin");
    eprintln!("    -t: terminal mode");
    std::process::exit(code);
}

fn main() {
    let mut terminal = false;
    let mut interactive = false;
    let mut args = Vec::new();
    let mut script = None;

    let args_raw: Vec<_> = std::env::args().collect();

    for idx in 1..args_raw.len() {
        let arg = args_raw[idx].clone();
        if idx == 1 {
            if arg.as_str() == "-t" {
                terminal = true;
                continue;
            }
            if arg.as_str() == "-i" {
                interactive = true;
                continue;
            }
            if arg.as_str() == "-h" {
                print_usage_and_exit(0);
            }
            if arg.as_str().starts_with('-') {
                print_usage_and_exit(1);
            }
        }

        if script.is_none() {
            script = Some(arg.clone());
        }
        args.push(arg);
    }

    if interactive {
        if let Some(script) = script {
            // This is usually config, setting PATH and such.
            run_script(script.as_str(), args, true).ok();
        }

        let mut parser = line_parser::LineParser::new();
        let args = vec![];
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if let Some(commands) = parser.parse_line(input.trim()) {
                exec::run(commands, false, &args).ok(); // Ignore results in the interactive mode.
            }
        }
        // unreachable
    }

    if let Some(script) = script {
        if !terminal {
            match run_script(script.as_str(), args, false) {
                Ok(()) => std::process::exit(0),
                Err(err) => std::process::exit(err),
            }
        }

        // This is usually config, setting PATH and such.
        run_script(script.as_str(), args, true).ok();
    }

    let is_terminal = std::io::IsTerminal::is_terminal(&std::io::stdin());
    if is_terminal && !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        // Is this even possible? If so, how should we behave?
        eprintln!("rush: stdin is a terminal, but stdiout is not.");
        std::process::exit(1)
    }

    if !is_terminal {
        eprintln!("rush: stdin is not a terminal.");
        std::process::exit(1)
    }

    if std::env::current_dir().is_err() {
        std::env::set_current_dir(std::path::Path::new("/")).unwrap();
    }

    let _cleanup = Cleanup {}; // On panic, restore the terminal state.
    let term = term::Term::new();
    let mut parser = line_parser::LineParser::new();

    let args = vec![];
    loop {
        if let Some(commands) = parser.parse_line(term.readline().as_str()) {
            exec::run(commands, false, &args).ok(); // Ignore results in the interactive mode.
        }
    }
}

pub fn exit(code: i32) -> ! {
    term::on_exit();
    std::process::exit(code)
}
