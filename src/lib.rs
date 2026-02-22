pub mod cli;
pub mod db;
pub mod ticket;

use rusqlite::Connection;
use ticket::AppError;

pub fn run(cli: cli::Cli, conn: Connection) -> Result<(), AppError> {
    use cli::Commands;
    match cli.command {
        Commands::Create(args) => {
            let desc = args.desc.as_deref().unwrap_or("");
            let id = ticket::create_ticket(&conn, &args.name, desc)?;
            println!("Created: #{} {}", id, args.name);
        }
        Commands::Get { id } => {
            let t = ticket::get_ticket(&conn, id)?;
            let created_date = t.created_at.split('T').next().unwrap_or(&t.created_at);
            let updated_date = t.updated_at.split('T').next().unwrap_or(&t.updated_at);
            println!("#{} {} [{}]", t.id, t.name, t.status);
            if !t.description.is_empty() {
                println!("{}", t.description);
            }
            println!("Created: {} | Updated: {}", created_date, updated_date);
        }
        Commands::Update(args) => {
            let name = args.name.as_deref();
            let desc = args.desc.as_deref();
            let status = args.status.as_deref();
            if name.is_none() && desc.is_none() && status.is_none() {
                eprintln!("Error: at least one field required (--name, --desc, --status)");
                std::process::exit(1);
            }
            let ticket_name = ticket::update_ticket(&conn, args.id, name, desc, status)?;
            println!("Updated: #{} {}", args.id, ticket_name);
        }
        Commands::Delete { id } => {
            let t = ticket::get_ticket(&conn, id)?;
            ticket::delete_ticket(&conn, id)?;
            println!("Deleted: #{} {}", id, t.name);
        }
        Commands::List(args) => {
            let tickets = ticket::list_tickets(&conn)?;
            if tickets.is_empty() {
                println!("No tickets.");
                return Ok(());
            }
            if args.timestamps {
                println!(
                    "{:>4}  {:<9}  {:<40}  {:<10}  {:<10}",
                    "ID", "STATUS", "NAME", "CREATED", "UPDATED"
                );
                println!("{}", "-".repeat(80));
                for t in &tickets {
                    let name = truncate_name(&t.name, 40);
                    let created = t.created_at.split('T').next().unwrap_or("");
                    let updated = t.updated_at.split('T').next().unwrap_or("");
                    println!(
                        "{:>4}  {:<9}  {:<40}  {:<10}  {:<10}",
                        t.id, t.status, name, created, updated
                    );
                }
            } else {
                println!("{:>4}  {:<9}  {}", "ID", "STATUS", "NAME");
                println!("{}", "-".repeat(60));
                for t in &tickets {
                    let name = truncate_name(&t.name, 40);
                    println!("{:>4}  {:<9}  {}", t.id, t.status, name);
                }
            }
        }
    }
    Ok(())
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() > max_len {
        format!("{}...", &name[..max_len - 3])
    } else {
        name.to_string()
    }
}
