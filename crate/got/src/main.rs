use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "goldest" => {
            eprintln!("goldest tool not yet implemented");
            std::process::exit(1);
        }
        "gotsel" => {
            eprintln!("gotsel tool not yet implemented");
            std::process::exit(1);
        }
        "statusd" => {
            eprintln!("statusd tool not yet implemented");
            std::process::exit(1);
        }
        "commitd" => {
            eprintln!("commitd tool not yet implemented");
            std::process::exit(1);
        }
        "nah" => {
            eprintln!("nah tool not yet implemented");
            std::process::exit(1);
        }
        _ => {
            eprintln!("unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("got - git tools your past self should have used");
    eprintln!();
    eprintln!("Usage: got <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  goldest   Find oldest changes and get datestamp");
    eprintln!("  gotsel    Git staging selection tree tool");
    eprintln!("  statusd   Status focused on modified time");
    eprintln!("  commitd   Commit using staged file dates");
    eprintln!("  nah       Ignore files");
    eprintln!();
    eprintln!("See README.md for more details");
}
