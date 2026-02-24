use rtik::{db, ticket};
use ticket::{ListFilter, TicketExport};

fn open_test_db() -> (rusqlite::Connection, tempfile::TempPath) {
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    let path = tmp.path().to_path_buf();
    let conn = db::open_connection(&path).expect("open_connection");
    (conn, tmp.into_temp_path())
}

fn empty_filter() -> ListFilter {
    ListFilter {
        status: None,
        claimed: None,
        claimer: None,
        search: vec![],
    }
}

// ---- Filter: status ----

#[test]
fn test_filter_by_status_returns_matching() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Todo ticket", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "In-progress ticket", "").unwrap();
    ticket::update_ticket(&conn, id2, None, None, Some("in-progress")).unwrap();

    let filter = ListFilter {
        status: Some("in-progress".to_string()),
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "In-progress ticket");
}

#[test]
fn test_filter_by_status_returns_empty_for_unknown() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Todo ticket", "").unwrap();

    let filter = ListFilter {
        status: Some("done".to_string()),
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert!(tickets.is_empty());
}

// ---- Filter: claimed / unclaimed ----

#[test]
fn test_filter_claimed_only() {
    let (mut conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Unclaimed", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "Claimed", "").unwrap();
    ticket::claim_ticket(&mut conn, id2, "agent-1", false).unwrap();

    let filter = ListFilter {
        claimed: Some(true),
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Claimed");
}

#[test]
fn test_filter_unclaimed_only() {
    let (mut conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "Unclaimed", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "Claimed", "").unwrap();
    ticket::claim_ticket(&mut conn, id2, "agent-1", false).unwrap();

    let filter = ListFilter {
        claimed: Some(false),
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].id, id1);
}

// ---- Filter: claimer ----

#[test]
fn test_filter_by_claimer() {
    let (mut conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "Alice task", "").unwrap();
    let id2 = ticket::create_ticket(&conn, "Bob task", "").unwrap();
    ticket::claim_ticket(&mut conn, id1, "alice", false).unwrap();
    ticket::claim_ticket(&mut conn, id2, "bob", false).unwrap();

    let filter = ListFilter {
        claimer: Some("alice".to_string()),
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Alice task");
}

// ---- Filter: search ----

#[test]
fn test_search_by_name() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Fix the foo bug", "").unwrap();
    ticket::create_ticket(&conn, "Update bar component", "").unwrap();

    let filter = ListFilter {
        search: vec!["foo".to_string()],
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Fix the foo bug");
}

#[test]
fn test_search_case_insensitive() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Fix the foo bug", "").unwrap();
    ticket::create_ticket(&conn, "Update bar component", "").unwrap();

    let filter = ListFilter {
        search: vec!["FOO".to_string()],
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Fix the foo bug");
}

#[test]
fn test_search_by_description() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Task one", "unique-desc-term here").unwrap();
    ticket::create_ticket(&conn, "Task two", "nothing special").unwrap();

    let filter = ListFilter {
        search: vec!["unique-desc-term".to_string()],
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Task one");
}

#[test]
fn test_search_multi_term_and() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Fix foo and bar", "").unwrap();
    ticket::create_ticket(&conn, "Fix foo only", "").unwrap();

    let filter = ListFilter {
        search: vec!["foo".to_string(), "bar".to_string()],
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Fix foo and bar");
}

#[test]
fn test_search_multi_term_no_match_partial() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "Fix foo only", "").unwrap();

    // Requires both "foo" AND "bar" — only has foo, should not match.
    let filter = ListFilter {
        search: vec!["foo".to_string(), "bar".to_string()],
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    assert!(tickets.is_empty());
}

// ---- Filter: composition ----

#[test]
fn test_filter_compose_status_and_search() {
    let (conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "Fix the router", "").unwrap();
    let _id2 = ticket::create_ticket(&conn, "Fix the login", "").unwrap();
    ticket::update_ticket(&conn, id1, None, None, Some("in-progress")).unwrap();

    let filter = ListFilter {
        status: Some("todo".to_string()),
        search: vec!["fix".to_string()],
        ..empty_filter()
    };
    let tickets = ticket::list_tickets_filtered(&conn, &filter).unwrap();
    // Only "Fix the login" is both todo and contains "fix".
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].name, "Fix the login");
}

// ---- Export: plain text ----

#[test]
fn test_export_plain_text_no_deps() {
    let export = TicketExport {
        id: 7,
        name: "Deploy service".to_string(),
        description: "".to_string(),
        status: "todo".to_string(),
        claimed_by: None,
        dependencies: vec![],
    };
    let text = ticket::format_export_text(&export);
    assert_eq!(text, "T-7 [todo] Deploy service");
}

#[test]
fn test_export_plain_text_with_deps() {
    let export = TicketExport {
        id: 3,
        name: "Integrate payments".to_string(),
        description: "".to_string(),
        status: "in-progress".to_string(),
        claimed_by: None,
        dependencies: vec![1, 2],
    };
    let text = ticket::format_export_text(&export);
    assert_eq!(text, "T-3 [in-progress] Integrate payments deps:T-1,T-2");
}

// ---- Export: JSON structure ----

#[test]
fn test_export_json_structure() {
    let (conn, _tmp) = open_test_db();
    let id1 = ticket::create_ticket(&conn, "Alpha", "first").unwrap();
    let id2 = ticket::create_ticket(&conn, "Beta", "second").unwrap();
    ticket::add_dep(&conn, id2, id1).unwrap();

    let exports = ticket::tickets_to_export(&conn, &empty_filter()).unwrap();
    let json = serde_json::to_string_pretty(&exports).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 2);
    assert!(parsed[0]["id"].is_number());
    assert!(parsed[0]["name"].is_string());
    assert!(parsed[0]["description"].is_string());
    assert!(parsed[0]["status"].is_string());
    assert!(parsed[0]["dependencies"].is_array());
    // Beta (id2) should have Alpha (id1) as dependency.
    let beta = &parsed[1];
    assert_eq!(beta["name"].as_str().unwrap(), "Beta");
    assert_eq!(beta["dependencies"].as_array().unwrap().len(), 1);
    assert_eq!(beta["dependencies"][0].as_i64().unwrap(), id1);
}

// ---- Empty filter returns all ----

#[test]
fn test_empty_filter_returns_all() {
    let (conn, _tmp) = open_test_db();
    ticket::create_ticket(&conn, "One", "").unwrap();
    ticket::create_ticket(&conn, "Two", "").unwrap();
    ticket::create_ticket(&conn, "Three", "").unwrap();

    let tickets = ticket::list_tickets_filtered(&conn, &empty_filter()).unwrap();
    assert_eq!(tickets.len(), 3);
}
