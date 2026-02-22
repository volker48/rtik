use clap::Parser;

fn main() {
    sigpipe::reset(); // CLI-06: must be first — resets SIGPIPE before any I/O

    let cli = rtik::cli::Cli::parse(); // exits with code 2 on usage error (CLI-05)
    let db_path = rtik::db::resolve_db_path();
    let conn = match rtik::db::open_connection(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: cannot open database: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = rtik::run(cli, conn) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
