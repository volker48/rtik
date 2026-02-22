use rtik::{db, ticket};

fn open_test_db() -> (rusqlite::Connection, tempfile::TempPath) {
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    let path = tmp.path().to_path_buf();
    let conn = db::open_connection(&path).expect("open_connection");
    (conn, tmp.into_temp_path())
}

#[test]
fn create_ticket_returns_incrementing_ids() {
    let (conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "First", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "Second", "").unwrap();
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn create_ticket_defaults_to_todo_status() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Buy milk", "Grocery").unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "todo");
    assert_eq!(t.name, "Buy milk");
    assert_eq!(t.description, "Grocery");
}

#[test]
fn get_ticket_not_found() {
    let (conn, _tmp) = open_test_db();
    let result = ticket::get_ticket(&conn, 999);
    assert!(matches!(result, Err(ticket::AppError::NotFound(999))));
}

#[test]
fn delete_ticket_removes_it() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Temp", "").unwrap();
    ticket::delete_ticket(&conn, id).unwrap();
    assert!(matches!(
        ticket::get_ticket(&conn, id),
        Err(ticket::AppError::NotFound(_))
    ));
}

#[test]
fn delete_nonexistent_ticket_returns_not_found() {
    let (conn, _tmp) = open_test_db();
    let result = ticket::delete_ticket(&conn, 999);
    assert!(matches!(result, Err(ticket::AppError::NotFound(999))));
}

#[test]
fn list_tickets_empty_db() {
    let (conn, _tmp) = open_test_db();
    let tickets = ticket::list_tickets(&conn).unwrap();
    assert!(tickets.is_empty());
}

#[test]
fn list_tickets_sorted_by_id() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Alpha", "").unwrap();
    ticket::create_ticket(&conn, "Beta", "").unwrap();
    let tickets = ticket::list_tickets(&conn).unwrap();
    assert_eq!(tickets.len(), 2);
    assert!(tickets[0].id < tickets[1].id);
    assert_eq!(tickets[0].name, "Alpha");
}

#[test]
fn update_ticket_status_normalized_to_lowercase() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("WIP")).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "wip");
}

#[test]
fn update_ticket_no_fields_returns_error() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    let result = ticket::update_ticket(&conn, id, None, None, None);
    assert!(matches!(result, Err(ticket::AppError::NoUpdateFields)));
}

#[test]
fn update_ticket_nonexistent_returns_not_found() {
    let (conn, _tmp) = open_test_db();
    let result = ticket::update_ticket(&conn, 999, Some("name"), None, None);
    assert!(matches!(result, Err(ticket::AppError::NotFound(999))));
}

#[test]
fn created_at_preserved_updated_at_changes_on_update() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    let before = ticket::get_ticket(&conn, id).unwrap();

    // Sleep 1 second to ensure timestamp difference
    std::thread::sleep(std::time::Duration::from_secs(1));

    ticket::update_ticket(&conn, id, Some("New name"), None, None).unwrap();
    let after = ticket::get_ticket(&conn, id).unwrap();

    assert_eq!(before.created_at, after.created_at, "created_at should not change");
    assert_ne!(before.updated_at, after.updated_at, "updated_at should change after update");
}

#[test]
fn update_ticket_partial_update_preserves_other_fields() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Original", "My desc").unwrap();
    ticket::update_ticket(&conn, id, Some("Renamed"), None, None).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.name, "Renamed");
    assert_eq!(t.description, "My desc"); // unchanged
    assert_eq!(t.status, "todo"); // unchanged
}

#[test]
fn created_at_is_iso8601_utc_format() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert!(t.created_at.contains('T'), "created_at should be ISO 8601 with T separator");
    assert!(t.created_at.ends_with('Z'), "created_at should end with Z (UTC)");
}
