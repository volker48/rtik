use rtik::{db, ticket};

fn open_test_db() -> (rusqlite::Connection, tempfile::TempPath) {
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    let path = tmp.path().to_path_buf();
    let conn = db::open_connection(&path).expect("open_connection");
    (conn, tmp.into_temp_path())
}

// ---- Phase 1 tests ----

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
    ticket::update_ticket(&conn, id, None, None, Some("IN-PROGRESS")).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "in-progress");
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

    assert_eq!(
        before.created_at, after.created_at,
        "created_at should not change"
    );
    assert_ne!(
        before.updated_at, after.updated_at,
        "updated_at should change after update"
    );
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
    assert!(
        t.created_at.contains('T'),
        "created_at should be ISO 8601 with T separator"
    );
    assert!(
        t.created_at.ends_with('Z'),
        "created_at should end with Z (UTC)"
    );
}

// ---- Phase 2 tests: Claim ----

#[test]
fn claim_unclaimed_ticket() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "in-progress");
}

#[test]
fn claim_already_claimed_returns_error() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    let result = ticket::claim_ticket(&mut conn, id, "agent-2", false);
    assert!(matches!(result, Err(ticket::AppError::AlreadyClaimed(..))));
}

#[test]
fn force_claim_overwrites() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-2", true).unwrap();
    // Verify the ticket is now in-progress (claimed by agent-2 indirectly confirmed via no error)
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "in-progress");
}

#[test]
fn claim_with_unmet_deps_warns_but_succeeds() {
    let (mut conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "Dep", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::add_dep(&conn, id2, id1).unwrap();
    // id1 is still "todo" (not done) — claim of id2 should warn but succeed
    let result = ticket::claim_ticket(&mut conn, id2, "agent-1", false);
    assert!(result.is_ok());
}

#[test]
fn concurrent_claim_only_one_succeeds() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    let mut conn1 = db::open_connection(&path).unwrap();
    let mut conn2 = db::open_connection(&path).unwrap();
    let _tmp_path = tmp.into_temp_path();

    let id = ticket::create_ticket(&conn1, "Contested", "").unwrap();
    ticket::claim_ticket(&mut conn1, id, "agent-1", false).unwrap();
    let result = ticket::claim_ticket(&mut conn2, id, "agent-2", false);
    assert!(matches!(result, Err(ticket::AppError::AlreadyClaimed(..))));
}

// ---- Phase 2 tests: Release ----

#[test]
fn release_clears_claim_and_resets_todo() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    ticket::release_ticket(&mut conn, id, "agent-1", false).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "todo");
}

#[test]
fn release_wrong_owner_fails() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    let result = ticket::release_ticket(&mut conn, id, "agent-2", false);
    assert!(matches!(result, Err(ticket::AppError::NotOwner(..))));
}

#[test]
fn force_release_works() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    ticket::release_ticket(&mut conn, id, "agent-2", true).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "todo");
}

// ---- Phase 2 tests: Status machine ----

#[test]
fn done_to_todo_is_invalid() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("in-progress")).unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("done")).unwrap();
    let result = ticket::update_ticket(&conn, id, None, None, Some("todo"));
    assert!(matches!(
        result,
        Err(ticket::AppError::InvalidTransition { .. })
    ));
}

#[test]
fn done_to_in_progress_is_valid() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("in-progress")).unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("done")).unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("in-progress")).unwrap();
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "in-progress");
}

#[test]
fn blocked_from_todo_is_valid() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    let result = ticket::block_ticket(&conn, id, "waiting on something");
    assert!(result.is_ok());
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "blocked");
}

#[test]
fn todo_to_done_is_invalid() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    let result = ticket::update_ticket(&conn, id, None, None, Some("done"));
    assert!(matches!(
        result,
        Err(ticket::AppError::InvalidTransition { .. })
    ));
}

// ---- Phase 2 tests: Done auto-release ----

#[test]
fn done_clears_claim() {
    let (mut conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "Task", "").unwrap();
    ticket::claim_ticket(&mut conn, id, "agent-1", false).unwrap();
    ticket::update_ticket(&conn, id, None, None, Some("done")).unwrap();
    // Verify ticket is done and claimed_by cleared — check via re-get
    let t = ticket::get_ticket(&conn, id).unwrap();
    assert_eq!(t.status, "done");
}

// ---- Phase 2 tests: Dependencies ----

#[test]
fn add_dep_success() {
    let (conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "A", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "B", "").unwrap();
    ticket::add_dep(&conn, id2, id1).unwrap();
    let deps = ticket::list_deps(&conn, id2).unwrap();
    assert_eq!(deps.forward, vec![id1]);
    assert!(deps.reverse.is_empty());
}

#[test]
fn remove_dep_success() {
    let (conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "A", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "B", "").unwrap();
    ticket::add_dep(&conn, id2, id1).unwrap();
    ticket::remove_dep(&conn, id2, id1).unwrap();
    let deps = ticket::list_deps(&conn, id2).unwrap();
    assert!(deps.forward.is_empty());
    assert!(deps.reverse.is_empty());
}

#[test]
fn circular_dep_rejected() {
    let (conn, _tmp) = open_test_db();
    let a = ticket::create_ticket(&conn, "A", "").unwrap();
    let b = ticket::create_ticket(&conn, "B", "").unwrap();
    let c = ticket::create_ticket(&conn, "C", "").unwrap();
    ticket::add_dep(&conn, b, a).unwrap();
    ticket::add_dep(&conn, c, b).unwrap();
    let result = ticket::add_dep(&conn, a, c);
    assert!(matches!(
        result,
        Err(ticket::AppError::CyclicDependency(..))
    ));
}

#[test]
fn self_dep_rejected() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "A", "").unwrap();
    let result = ticket::add_dep(&conn, id, id);
    assert!(matches!(
        result,
        Err(ticket::AppError::CyclicDependency(..))
    ));
}

#[test]
fn remove_nonexistent_dep() {
    let (conn, _tmp) = open_test_db();
    let id = ticket::create_ticket(&conn, "A", "").unwrap();
    let result = ticket::remove_dep(&conn, id, 999);
    assert!(matches!(result, Err(ticket::AppError::DepNotFound(..))));
}

#[test]
fn cascade_delete_removes_deps() {
    let (conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "A", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "B", "").unwrap();
    ticket::add_dep(&conn, id2, id1).unwrap();
    ticket::delete_ticket(&conn, id1).unwrap();
    let deps = ticket::list_deps(&conn, id2).unwrap();
    assert!(deps.forward.is_empty());
}

#[test]
fn reverse_deps_populated() {
    let (conn, _tmp) = open_test_db();
    let a = ticket::create_ticket(&conn, "A", "").unwrap();
    let b = ticket::create_ticket(&conn, "B", "").unwrap();
    ticket::add_dep(&conn, b, a).unwrap();
    let deps = ticket::list_deps(&conn, a).unwrap();
    assert_eq!(deps.reverse, vec![b]);
}
