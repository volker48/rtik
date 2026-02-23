pub mod cli;
pub mod db;
pub mod ticket;

use rusqlite::Connection;
use ticket::AppError;

fn resolve_agent() -> Result<String, AppError> {
    std::env::var("RTIK_AGENT").map_err(|_| AppError::AgentNotSet)
}

pub fn run(cli: cli::Cli, conn: Connection) -> Result<(), AppError> {
    let mut conn = conn;
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
            let deps = ticket::list_deps(&conn, id)?;
            if !deps.forward.is_empty() {
                let fwd = deps.forward.iter().map(|i| format!("#{}", i)).collect::<Vec<_>>().join(", ");
                println!("Depends on: {}", fwd);
            }
            if !deps.reverse.is_empty() {
                let rev = deps.reverse.iter().map(|i| format!("#{}", i)).collect::<Vec<_>>().join(", ");
                println!("Required by: {}", rev);
            }
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
            let dep_counts = load_dep_counts(&conn)?;
            if args.timestamps {
                println!(
                    "{:>4}  {:<9}  {:<40}  {:<10}  {:<10}",
                    "ID", "STATUS", "NAME", "CREATED", "UPDATED"
                );
                println!("{}", "-".repeat(80));
                for t in &tickets {
                    let name = format_name_with_deps(&t.name, dep_counts.get(&t.id).copied());
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
                    let name = format_name_with_deps(&t.name, dep_counts.get(&t.id).copied());
                    println!("{:>4}  {:<9}  {}", t.id, t.status, name);
                }
            }
        }
        Commands::Claim(args) => {
            let agent = resolve_agent()?;
            ticket::claim_ticket(&mut conn, args.id, &agent, args.force)?;
            println!("Claimed #{}", args.id);
        }
        Commands::Release(args) => {
            let agent = resolve_agent()?;
            ticket::release_ticket(&mut conn, args.id, &agent, args.force)?;
            println!("Released #{}", args.id);
        }
        Commands::Block(args) => {
            let name = ticket::block_ticket(&conn, args.id, &args.reason)?;
            println!("Blocked: #{} {}", args.id, name);
        }
        Commands::Dep(args) => {
            match args.action {
                cli::DepAction::Add { ticket_id, dep_id } => {
                    ticket::add_dep(&conn, ticket_id, dep_id)?;
                    println!("Added: #{} depends on #{}", ticket_id, dep_id);
                }
                cli::DepAction::Remove { ticket_id, dep_id } => {
                    ticket::remove_dep(&conn, ticket_id, dep_id)?;
                    println!("Removed: #{} no longer depends on #{}", ticket_id, dep_id);
                }
            }
        }
        Commands::Deps(args) => {
            let deps = ticket::list_deps(&conn, args.id)?;
            if deps.forward.is_empty() && deps.reverse.is_empty() {
                println!("#{} has no dependencies.", args.id);
            } else {
                if !deps.forward.is_empty() {
                    let fwd = deps
                        .forward
                        .iter()
                        .map(|i| format!("#{}", i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("Depends on: {}", fwd);
                }
                if !deps.reverse.is_empty() {
                    let rev = deps
                        .reverse
                        .iter()
                        .map(|i| format!("#{}", i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("Required by: {}", rev);
                }
            }
        }
        Commands::Export(_args) => {
            eprintln!("export: not yet implemented");
            std::process::exit(1);
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

fn format_name_with_deps(name: &str, dep_count: Option<i64>) -> String {
    match dep_count {
        Some(n) if n > 0 => {
            let suffix = format!(" [{} dep{}]", n, if n == 1 { "" } else { "s" });
            let truncated = truncate_name(name, 35);
            format!("{}{}", truncated, suffix)
        }
        _ => truncate_name(name, 40),
    }
}

fn load_dep_counts(conn: &Connection) -> Result<std::collections::HashMap<i64, i64>, ticket::AppError> {
    let mut stmt = conn.prepare(
        "SELECT ticket_id, COUNT(*) FROM ticket_deps GROUP BY ticket_id",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (id, count) = row?;
        map.insert(id, count);
    }
    Ok(map)
}
