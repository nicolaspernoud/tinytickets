use crate::config::Config;
use crate::mail::Mailer;
use crate::models::asset::{Asset, InAsset};
use crate::models::comment::{Comment, InComment};
use crate::models::ticket::{InTicket, Ticket};
use rocket::http::Header;
use std::env;
use std::fs;
use std::path::Path;

use chrono::NaiveDateTime;

use rocket::http::Status;
use rocket::local::blocking::Client;
use std::convert::TryFrom;

fn test_assets(base: &str, client: &rocket::local::blocking::Client) {
    // Number of assets we're going to create/read/delete.
    const N: usize = 20;
    let admin_header = Header::new("X-TOKEN", "$ADMIN$development_admin_token");
    let user_header = Header::new("X-TOKEN", "$USER$development_user_token");

    // Clear everything from the database.
    assert_eq!(
        client.delete(base).dispatch().status(),
        Status::Unauthorized
    );
    assert_eq!(
        client
            .delete(base)
            .header(user_header.clone())
            .dispatch()
            .status(),
        Status::Forbidden
    );
    assert_eq!(
        client
            .delete(base)
            .header(admin_header.clone())
            .dispatch()
            .status(),
        Status::Ok
    );
    assert_eq!(
        client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>(),
        Some(vec![])
    );

    // Add some random assets, ensure they're listable and readable.
    for i in 1..=N {
        let title = format!("My Asset - {}", i);
        let description = format!("Once upon a time, at {}'o clock...", i);
        let asset = InAsset {
            title: title.clone(),
            description: description.clone(),
        };

        // Create a new asset.
        assert_eq!(
            client.post(base).json(&asset).dispatch().status(),
            Status::Unauthorized
        );

        let response = client
            .post(base)
            .header(admin_header.clone())
            .json(&asset)
            .dispatch()
            .into_json::<InAsset>();
        assert_eq!(response.unwrap(), asset);

        // Ensure the index shows one more asset.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client
            .get(format!("{}/{}", base, last))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.into_json::<Asset>().unwrap(), asset);
    }

    // Patch the assets
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        let id = list.get(0).expect("have asset");

        // Patch that asset.
        let asset = Asset {
            id: i32::try_from(*id).unwrap(),
            title: "patched title".to_string(),
            description: format!("Once upon a time, at {}'o clock...", id),
        };
        assert_eq!(
            client
                .patch(format!("{}/{}", base, id))
                .json(&asset)
                .dispatch()
                .status(),
            Status::Unauthorized
        );
        let response = client
            .patch(format!("{}/{}", base, id))
            .header(admin_header.clone())
            .json(&asset)
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        // Check that asset is patched
        let response = client
            .get(format!("{}/{}", base, id))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.into_json::<Asset>().unwrap(), asset);
    }

    // Now delete all of the assets.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        let id = list.get(0).expect("have asset");

        // Delete that asset.
        assert_eq!(
            client
                .delete(format!("{}/{}", base, id))
                .dispatch()
                .status(),
            Status::Unauthorized
        );
        assert_eq!(
            client
                .delete(format!("{}/{}", base, id))
                .header(admin_header.clone())
                .dispatch()
                .status(),
            Status::Ok
        );
    }

    // Ensure they're all gone.
    let list = client
        .get(base)
        .header(user_header.clone())
        .dispatch()
        .into_json::<Vec<i64>>()
        .unwrap();
    assert!(list.is_empty());

    // Trying to delete should now 404.
    let response = client
        .delete(format!("{}/{}", base, 1))
        .header(admin_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

fn test_tickets(base: &str, client: &rocket::local::blocking::Client) {
    // Number of tickets we're going to create/read/delete.
    const N: usize = 20;
    let admin_header = Header::new("X-TOKEN", "$ADMIN$development_admin_token");
    let user_header = Header::new("X-TOKEN", "$USER$development_user_token");

    // Clear everything from the database.
    assert_eq!(
        client.delete(base).dispatch().status(),
        Status::Unauthorized
    );
    assert_eq!(
        client
            .delete(base)
            .header(user_header.clone())
            .dispatch()
            .status(),
        Status::Forbidden
    );
    assert_eq!(
        client
            .delete(base)
            .header(admin_header.clone())
            .dispatch()
            .status(),
        Status::Ok
    );
    assert_eq!(
        client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>(),
        Some(vec![])
    );

    // Add a ticket to a non existing asset (must fail)
    let ticket = InTicket {
        title: "test".to_string(),
        creator: "test".to_string(),
        creator_mail: "test".to_string(),
        creator_phone: "test".to_string(),
        description: "test".to_string(),
        time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        asset_id: 1,
        is_closed: false,
    };
    // Create a new ticket.
    let response = client
        .post(base)
        .header(user_header.clone())
        .json(&ticket)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);

    // Add an asset
    let asset = InAsset {
        title: "MyAsset".to_string(),
        description: "MyAssetDescription".to_string(),
    };
    let response = client
        .post("/api/assets")
        .header(admin_header.clone())
        .json(&asset)
        .dispatch()
        .into_json::<InAsset>()
        .unwrap();
    assert_eq!(response, asset);

    // Get a valid asset id
    let asset_id = client
        .get("/api/assets")
        .header(user_header.clone())
        .dispatch()
        .into_json::<Vec<i32>>()
        .unwrap()[0];

    // Add some random tickets, ensure they're listable and readable.
    for i in 1..=N {
        let title = format!("My Ticket - {}", i);
        let creator = format!("My Ticket Creator - {}", i);
        let creator_mail = format!("My Ticket Creator Mail - {}", i);
        let creator_phone = format!("My Ticket Creator Tel - {}", i);
        let description = format!("Once upon a time, at {}'o clock...", i);
        let ticket = InTicket {
            title: title.clone(),
            creator: creator.clone(),
            creator_mail: creator_mail.clone(),
            creator_phone: creator_phone.clone(),
            description: description.clone(),
            time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            asset_id: asset_id,
            is_closed: false,
        };

        assert_eq!(
            client.post(base).json(&ticket).dispatch().status(),
            Status::Unauthorized
        );

        // Create a new ticket.
        let response = client
            .post(base)
            .json(&ticket)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Ticket>();
        assert_eq!(response.unwrap(), ticket);

        // Ensure the index shows one more ticket.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client
            .get(format!("{}/{}", base, last))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.into_json::<Ticket>().unwrap(), ticket);
    }

    // Patch the tickets
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        let id = list.get(0).expect("have ticket");

        // Patch that ticket.
        let ticket = Ticket {
            id: i32::try_from(*id).unwrap(),
            title: "patched title".to_string(),
            creator: "patched creator".to_string(),
            creator_mail: "patched creator mail".to_string(),
            creator_phone: "patched creator tel".to_string(),
            description: format!("Once upon a time, at {}'o clock...", id),
            time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            asset_id: asset_id,
            is_closed: true,
        };
        let response = client
            .patch(format!("{}/{}", base, id))
            .json(&ticket)
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let response = client
            .patch(format!("{}/{}", base, id))
            .header(admin_header.clone())
            .json(&ticket)
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        // Check that ticket is patched
        let response = client
            .get(format!("{}/{}", base, id))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.into_json::<Ticket>().unwrap(), ticket);
    }

    // Test a photo upload without a user token
    let response = client.post(format!("{}/photos/{}", base, 1)).dispatch();
    assert_eq!(response.status(), Status::Unauthorized);

    // Test a photo upload with a user token
    let img_body = fs::read("test_img.jpg").unwrap();
    let response = client
        .post(format!("{}/photos/{}", base, 1))
        .body(&img_body)
        .header(user_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Test a photo retrieval with a user token
    let response = client
        .get(format!("{}/photos/{}", base, 1))
        .header(user_header.clone())
        .dispatch();
    assert_eq!(response.into_bytes().unwrap(), img_body);

    // Now delete all of the tickets.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        let id = list.get(0).expect("have ticket");

        // Delete that ticket.
        let response = client
            .delete(format!("{}/{}", base, id))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let response = client
            .delete(format!("{}/{}", base, id))
            .header(admin_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    // Ensure they're all gone.
    let list = client
        .get(base)
        .header(user_header.clone())
        .dispatch()
        .into_json::<Vec<i64>>()
        .unwrap();
    assert!(list.is_empty());

    // Check that the photo is gone too
    let response = client
        .get(format!("{}/photos/{}", base, 1))
        .header(user_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);

    // Trying to delete should now 404.
    let response = client
        .delete(format!("{}/{}", base, 1))
        .header(admin_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

fn test_comments(base: &str, client: &rocket::local::blocking::Client) {
    // Number of comments we're going to create/read/delete.
    const N: usize = 20;
    let admin_header = Header::new("X-TOKEN", "$ADMIN$development_admin_token");
    let user_header = Header::new("X-TOKEN", "$USER$development_user_token");

    // Clear everything from the database.
    assert_eq!(
        client.delete(base).dispatch().status(),
        Status::Unauthorized
    );
    assert_eq!(
        client
            .delete(base)
            .header(user_header.clone())
            .dispatch()
            .status(),
        Status::Forbidden
    );
    assert_eq!(
        client
            .delete(base)
            .header(admin_header.clone())
            .dispatch()
            .status(),
        Status::Ok
    );
    assert_eq!(
        client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>(),
        Some(vec![])
    );

    // Add a comment to a non existing ticket (must fail)
    let comment = InComment {
        creator: "test".to_string(),
        content: "test".to_string(),
        time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        ticket_id: 1,
    };
    // Create a new comment.
    let response = client
        .post(base)
        .header(user_header.clone())
        .json(&comment)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);

    // Add an asset
    let asset = InAsset {
        title: "MyAsset".to_string(),
        description: "MyAssetDescription".to_string(),
    };
    let response = client
        .post("/api/assets")
        .header(admin_header.clone())
        .json(&asset)
        .dispatch()
        .into_json::<InAsset>()
        .unwrap();
    assert_eq!(response, asset);

    // Get a valid asset id
    let asset_id = client
        .get("/api/assets")
        .header(user_header.clone())
        .dispatch()
        .into_json::<Vec<i32>>()
        .unwrap()[0];

    // Add a ticket
    let ticket = InTicket {
        title: "MyTicket".to_string(),
        creator: "MyTicketCreator".to_string(),
        creator_mail: "MyTicketCreatorMail".to_string(),
        creator_phone: "MyTicketCreatorTel".to_string(),
        description: "MyDescription".to_string(),
        time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        asset_id: asset_id,
        is_closed: false,
    };
    let response = client
        .post("/api/tickets")
        .header(user_header.clone())
        .json(&ticket)
        .dispatch()
        .into_json::<InTicket>()
        .unwrap();
    assert_eq!(response, ticket);

    // Get a valid ticket id
    let ticket_id = client
        .get("/api/tickets")
        .header(user_header.clone())
        .dispatch()
        .into_json::<Vec<i32>>()
        .unwrap()[0];

    // Add some random comments, ensure they're listable and readable.
    for i in 1..=N {
        let comment = InComment {
            ticket_id: ticket_id,
            creator: format!("My Comment Creator - {}", i),
            content: format!("My Comment - {}", i),
            time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
        };

        assert_eq!(
            client.post(base).json(&ticket).dispatch().status(),
            Status::Unauthorized
        );

        // Create a new comment.
        let response = client
            .post(base)
            .header(user_header.clone())
            .json(&comment)
            .dispatch()
            .into_json::<InComment>();
        assert_eq!(response.unwrap(), comment);

        // Ensure the index shows one more comment.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client
            .get(format!("{}/{}", base, last))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.into_json::<Comment>().unwrap(), comment);
    }

    // Patch the comments
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        let id = list.get(0).expect("have comment");

        // Patch that comment.
        let comment = Comment {
            id: i32::try_from(*id).unwrap(),
            creator: "patched creator".to_string(),
            content: "patched content".to_string(),
            time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            ticket_id: ticket_id,
        };
        let response = client
            .patch(format!("{}/{}", base, id))
            .json(&ticket)
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let response = client
            .patch(format!("{}/{}", base, id))
            .json(&ticket)
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let response = client
            .patch(format!("{}/{}", base, id))
            .header(admin_header.clone())
            .json(&comment)
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        // Check that comment is patched
        let response = client
            .get(format!("{}/{}", base, id))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.into_json::<Comment>().unwrap(), comment);
    }

    // Now delete all of the comments.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .header(user_header.clone())
            .dispatch()
            .into_json::<Vec<i64>>()
            .unwrap();
        let id = list.get(0).expect("have comment");

        let response = client
            .delete(format!("{}/{}", base, id))
            .header(user_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let response = client
            .delete(format!("{}/{}", base, id))
            .header(admin_header.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
    // Ensure they're all gone.
    let list = client
        .get(base)
        .header(user_header.clone())
        .dispatch()
        .into_json::<Vec<i64>>()
        .unwrap();
    assert!(list.is_empty());

    // Trying to delete should now 404.
    let response = client
        .delete(format!("{}/{}", base, 1))
        .header(admin_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_models() {
    // Remove the db to start fresh
    if Path::new("db/db.sqlite").exists() {
        if let Err(e) = fs::remove_file("db/db.sqlite") {
            panic!("error removing db: {}", e);
        }
    }
    env::set_var("ADMIN_TOKEN", "development_admin_token");
    env::set_var("USER_TOKEN", "development_user_token");
    // NOTE: If we had more than one test running concurrently that dispatches
    // DB-accessing requests, we'd need transactions or to serialize all tests.
    let mailer = Mailer::new(true);
    let client = Client::tracked(
        rocket::build()
            .attach(crate::models::stage())
            .manage(Config::init())
            .manage(mailer.clone()),
    )
    .unwrap();

    test_assets("/api/assets", &client);
    test_tickets("/api/tickets", &client);
    test_comments("/api/comments", &client);
    assert!(mailer
        .print_test_mails()
        .contains("Ticket created by patched creator: patched title has been closed"));
}
