use std::env;
use std::fs;
use std::path::Path;

use http::HeaderMap;
use http::StatusCode;
use tinytickets_backend::mail::Mailer;

use chrono::NaiveDateTime;
use tinytickets_backend::models::asset::Asset;
use tinytickets_backend::models::asset::InAsset;
use tinytickets_backend::models::comment::Comment;
use tinytickets_backend::models::comment::InComment;
use tinytickets_backend::models::ticket::InTicket;
use tinytickets_backend::models::ticket::Ticket;

use std::convert::TryFrom;

async fn test_assets(base: &str, client: &reqwest::Client) {
    // Number of assets we're going to create/read/delete.
    const N: usize = 20;
    let (admin_header, user_header) = headers();

    // Clear everything from the database.
    assert_eq!(
        client.delete(base).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .delete(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        client
            .delete(base)
            .headers(admin_header.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Option<Vec<i64>>>()
            .await
            .unwrap(),
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
            client
                .post(base)
                .json(&asset)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );

        let response = client
            .post(base)
            .headers(admin_header.clone())
            .json(&asset)
            .send()
            .await
            .unwrap()
            .json::<InAsset>()
            .await
            .unwrap();
        assert_eq!(response, asset);

        // Ensure the index shows one more asset.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client
            .get(format!("{}/{}", base, last))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Asset>()
            .await
            .unwrap();
        assert_eq!(response, asset);
    }

    // Patch the assets
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
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
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let response = client
            .patch(format!("{}/{}", base, id))
            .headers(admin_header.clone())
            .json(&asset)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        // Check that asset is patched
        let response = client
            .get(format!("{}/{}", base, id))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Asset>()
            .await
            .unwrap();
        assert_eq!(response, asset);
    }

    // Now delete all of the assets.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        let id = list.get(0).expect("have asset");

        // Delete that asset.
        assert_eq!(
            client
                .delete(format!("{}/{}", base, id))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            client
                .delete(format!("{}/{}", base, id))
                .headers(admin_header.clone())
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }

    // Ensure they're all gone.
    let list = client
        .get(base)
        .headers(user_header.clone())
        .send()
        .await
        .unwrap()
        .json::<Vec<i64>>()
        .await
        .unwrap();
    assert!(list.is_empty());

    // Trying to delete should now 404.
    let response = client
        .delete(format!("{}/{}", base, 1))
        .headers(admin_header.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

fn headers() -> (HeaderMap, HeaderMap) {
    let mut admin_header = HeaderMap::new();
    admin_header.insert("X-TOKEN", "$ADMIN$development_admin_token".parse().unwrap());
    let mut user_header = HeaderMap::new();
    user_header.insert("X-TOKEN", "$USER$development_user_token".parse().unwrap());
    (admin_header, user_header)
}

async fn test_tickets(base: &str, client: &reqwest::Client) {
    // Number of tickets we're going to create/read/delete.
    const N: usize = 20;
    let (admin_header, user_header) = headers();

    // Clear everything from the database.
    assert_eq!(
        client.delete(base).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .delete(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        client
            .delete(base)
            .headers(admin_header.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Option<Vec<i64>>>()
            .await
            .unwrap(),
        Some(vec![])
    );

    // Add a ticket to a non existing asset (must fail)
    let ticket = InTicket {
        title: "test".to_string(),
        creator: "test".to_string(),
        creator_mail: "test@test.com".to_string(),
        creator_phone: "0102030405".to_string(),
        description: "test".to_string(),
        time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        asset_id: 1,
        is_closed: false,
    };
    // Create a new ticket.
    let response = client
        .post(base)
        .headers(user_header.clone())
        .json(&ticket)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Add an asset
    let asset = InAsset {
        title: "MyAsset".to_string(),
        description: "MyAssetDescription".to_string(),
    };
    let response = client
        .post("/api/assets")
        .headers(admin_header.clone())
        .json(&asset)
        .send()
        .await
        .unwrap()
        .json::<InAsset>()
        .await
        .unwrap();
    assert_eq!(response, asset);

    // Get a valid asset id
    let asset_id = client
        .get("/api/assets")
        .headers(user_header.clone())
        .send()
        .await
        .unwrap()
        .json::<Vec<i32>>()
        .await
        .unwrap()[0];

    // Add some random tickets, ensure they're listable and readable.
    for i in 1..=N {
        let title = format!("My Ticket - {}", i);
        let creator = format!("My Ticket Creator - {}", i);
        let creator_mail = format!("testmail{}@test.com", i);
        let creator_phone = format!("000000{}", i);
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
            client
                .post(base)
                .json(&ticket)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );

        // Create a new ticket.
        let response = client
            .post(base)
            .json(&ticket)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Ticket>()
            .await;
        assert_eq!(response.unwrap(), ticket);

        // Ensure the index shows one more ticket.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client
            .get(format!("{}/{}", base, last))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.json::<Ticket>().await.unwrap(), ticket);
    }

    // Patch the tickets
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        let id = list.get(0).expect("have ticket");

        // Patch that ticket.
        let ticket = Ticket {
            id: i32::try_from(*id).unwrap(),
            title: "patched title".to_string(),
            creator: "patched creator".to_string(),
            creator_mail: "patchedmail@test.com".to_string(),
            creator_phone: "010203040506".to_string(),
            description: format!("Once upon a time, at {}'o clock...", id),
            time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            asset_id: asset_id,
            is_closed: true,
        };
        let response = client
            .patch(format!("{}/{}", base, id))
            .json(&ticket)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = client
            .patch(format!("{}/{}", base, id))
            .headers(admin_header.clone())
            .json(&ticket)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        // Check that ticket is patched
        let response = client
            .get(format!("{}/{}", base, id))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.json::<Ticket>().await.unwrap(), ticket);
    }

    // Test a photo upload without a user token
    let response = client
        .post(format!("{}/photos/{}", base, 1))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test a photo upload with a user token
    let img_body = fs::read("test_img.jpg").unwrap();
    let response = client
        .post(format!("{}/photos/{}", base, 1))
        .body(img_body.clone())
        .headers(user_header.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test a photo retrieval with a user token
    let response = client
        .get(format!("{}/photos/{}", base, 1))
        .headers(user_header.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.bytes().await.unwrap(), img_body);

    // Now delete all of the tickets.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        let id = list.get(0).expect("have ticket");

        // Delete that ticket.
        let response = client
            .delete(format!("{}/{}", base, id))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = client
            .delete(format!("{}/{}", base, id))
            .headers(admin_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Ensure they're all gone.
    let list = client
        .get(base)
        .headers(user_header.clone())
        .send()
        .await
        .unwrap()
        .json::<Vec<i64>>()
        .await
        .unwrap();
    assert!(list.is_empty());

    // Check that the photo is gone too
    let response = client
        .get(format!("{}/photos/{}", base, 1))
        .headers(user_header.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Trying to delete should now 404.
    let response = client
        .delete(format!("{}/{}", base, 1))
        .headers(admin_header.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn test_comments(base: &str, client: &reqwest::Client) {
    // Number of comments we're going to create/read/delete.
    const N: usize = 20;
    let (admin_header, user_header) = headers();

    // Clear everything from the database.
    assert_eq!(
        client.delete(base).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .delete(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        client
            .delete(base)
            .headers(admin_header.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Option<Vec<i64>>>()
            .await
            .unwrap(),
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
        .headers(user_header.clone())
        .json(&comment)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Add an asset
    let asset = InAsset {
        title: "MyAsset".to_string(),
        description: "MyAssetDescription".to_string(),
    };
    let response = client
        .post("/api/assets")
        .headers(admin_header.clone())
        .json(&asset)
        .send()
        .await
        .unwrap()
        .json::<InAsset>()
        .await
        .unwrap();
    assert_eq!(response, asset);

    // Get a valid asset id
    let asset_id = client
        .get("/api/assets")
        .headers(user_header.clone())
        .send()
        .await
        .unwrap()
        .json::<Vec<i32>>()
        .await
        .unwrap()[0];

    // Add a ticket
    let ticket = InTicket {
        title: "MyTicket".to_string(),
        creator: "MyTicketCreator".to_string(),
        creator_mail: "test@test.com".to_string(),
        creator_phone: "01020304".to_string(),
        description: "MyDescription".to_string(),
        time: NaiveDateTime::parse_from_str("2021-08-12T20:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        asset_id: asset_id,
        is_closed: false,
    };
    let response = client
        .post("/api/tickets")
        .headers(user_header.clone())
        .json(&ticket)
        .send()
        .await
        .unwrap()
        .json::<InTicket>()
        .await
        .unwrap();
    assert_eq!(response, ticket);

    // Get a valid ticket id
    let ticket_id = client
        .get("/api/tickets")
        .headers(user_header.clone())
        .send()
        .await
        .unwrap()
        .json::<Vec<i32>>()
        .await
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
            client
                .post(base)
                .json(&ticket)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );

        // Create a new comment.
        let response = client
            .post(base)
            .headers(user_header.clone())
            .json(&comment)
            .send()
            .await
            .unwrap()
            .json::<InComment>()
            .await
            .unwrap();
        assert_eq!(response, comment);

        // Ensure the index shows one more comment.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client
            .get(format!("{}/{}", base, last))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Comment>()
            .await
            .unwrap();
        assert_eq!(response, comment);
    }

    // Patch the comments
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
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
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = client
            .patch(format!("{}/{}", base, id))
            .json(&ticket)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = client
            .patch(format!("{}/{}", base, id))
            .headers(admin_header.clone())
            .json(&comment)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        // Check that comment is patched
        let response = client
            .get(format!("{}/{}", base, id))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Comment>()
            .await
            .unwrap();
        assert_eq!(response, comment);
    }

    // Now delete all of the comments.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client
            .get(base)
            .headers(user_header.clone())
            .send()
            .await
            .unwrap()
            .json::<Vec<i64>>()
            .await
            .unwrap();
        let id = list.get(0).expect("have comment");

        let response = client
            .delete(format!("{}/{}", base, id))
            .headers(user_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = client
            .delete(format!("{}/{}", base, id))
            .headers(admin_header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    // Ensure they're all gone.
    let list = client
        .get(base)
        .headers(user_header.clone())
        .send()
        .await
        .unwrap()
        .json::<Vec<i64>>()
        .await
        .unwrap();
    assert!(list.is_empty());

    // Trying to delete should now 404.
    let response = client
        .delete(format!("{}/{}", base, 1))
        .headers(admin_header.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_models() {
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
    let client = reqwest::Client::builder().build().unwrap();

    test_assets("/api/assets", &client).await;
    test_tickets("/api/tickets", &client).await;
    test_comments("/api/comments", &client).await;
    assert!(mailer
        .print_test_mails()
        .contains("Ticket created by patched creator: patched title has been closed"));
}
