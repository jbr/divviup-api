use test_support::*;

mod index {
    use super::{assert_eq, test, *};
    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let _other_account = fixtures::account(&app).await;

        let mut conn = get("/api/accounts")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let accounts: Vec<Account> = conn.response_json().await;
        assert_eq!(accounts, vec![account]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_admin(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::admin(&app).await;
        let other_account = fixtures::account(&app).await;
        let mut conn = get("/api/accounts")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        let accounts: Vec<Account> = conn.response_json().await;
        assert_eq!(accounts, vec![account, other_account]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_admin_token(app: DivviupApi) -> TestResult {
        let account = fixtures::admin_account(&app).await;
        let other_account = fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let mut conn = get("/api/accounts")
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .run_async(&app)
            .await;
        let accounts: Vec<Account> = conn.response_json().await;

        assert_eq!(accounts, vec![account, other_account]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_normal_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let mut conn = get("/api/accounts")
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .run_async(&app)
            .await;
        let accounts: Vec<Account> = conn.response_json().await;

        assert_eq!(accounts, vec![account]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_bogus_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        fixtures::account(&app).await;
        let (token, header) = fixtures::api_token(&app, &account).await;
        token.delete(app.db()).await?;
        let conn = get("/api/accounts")
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .run_async(&app)
            .await;
        assert_response!(conn, 403);
        Ok(())
    }
}

mod show {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_a_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let mut conn = get(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let account_response: Account = conn.response_json().await;
        assert_eq!(account_response, account);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let other_account = fixtures::account(&app).await;
        let mut conn = get(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member_but_as_an_admin(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::admin(&app).await;
        let other_account = fixtures::account(&app).await;

        let mut conn = get(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let account: Account = conn.response_json().await;

        assert_eq!(account, other_account);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_admin_token(app: DivviupApi) -> TestResult {
        let account = fixtures::admin_account(&app).await;
        let other_account = fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let mut conn = get(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .run_async(&app)
            .await;
        let response: Account = conn.response_json().await;

        assert_eq!(response, other_account);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_normal_token_with_access(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let mut conn = get(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .run_async(&app)
            .await;

        let response: Account = conn.response_json().await;
        assert_eq!(response, account);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_normal_token_without_access(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let other_account = fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let mut conn = get(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }
}

mod create {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn not_logged_in(app: DivviupApi) -> TestResult {
        let conn = post("/api/accounts")
            .with_api_headers()
            .with_request_json(json!({ "name": "some account name" }))
            .run_async(&app)
            .await;

        assert_response!(conn, 403);
        let accounts = Accounts::find().all(app.db()).await?;
        assert_eq!(accounts.len(), 0);
        let memberships = Memberships::find().all(app.db()).await?;
        assert_eq!(memberships.len(), 0);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn valid(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = post("/api/accounts")
            .with_api_headers()
            .with_state(user.clone())
            .with_request_json(json!({ "name": "some account name" }))
            .run_async(&app)
            .await;
        assert_response!(conn, 202);
        let account: Account = conn.response_json().await;
        assert_eq!(account.name, "some account name");

        let accounts = Accounts::find().all(app.db()).await?;

        assert_eq!(accounts, [account.clone()]);

        let memberships = Memberships::find().all(app.db()).await?;
        assert_eq!(memberships.len(), 1);
        assert_eq!(&memberships[0].user_email, &user.email);
        assert_eq!(&memberships[0].account_id, &account.id);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = post("/api/accounts")
            .with_api_headers()
            .with_state(user.clone())
            .with_request_json(json!({ "name": "" }))
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("name").is_some());
        let accounts = Accounts::find().all(app.db()).await?;
        assert_eq!(accounts.len(), 0);
        let memberships = Memberships::find().all(app.db()).await?;
        assert_eq!(memberships.len(), 0);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let name = fixtures::random_name();
        let mut conn = post("/api/accounts")
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(json!({ "name": name }))
            .run_async(&app)
            .await;
        assert_response!(conn, 202);
        let account: Account = conn.response_json().await;
        assert_eq!(&account.name, &name);
        assert_eq!(account.reload(app.db()).await?.unwrap().name, name);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonadmin_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let conn = post("/api/accounts")
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(json!({ "name": name }))
            .run_async(&app)
            .await;

        assert_response!(conn, 403);

        Ok(())
    }
}

mod update {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_a_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = patch(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new name" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 202);
        let response: Account = conn.response_json().await;
        assert_eq!(&response.name, "new name");
        assert_eq!(account.reload(app.db()).await?.unwrap().name, "new name");

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let other_account = fixtures::account(&app).await;
        let mut conn = patch(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new name" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member_but_as_an_admin(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::admin(&app).await;
        let other_account = fixtures::account(&app).await;

        let mut conn = patch(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new name" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 202);
        let account: Account = conn.response_json().await;

        assert_eq!(&account.name, "new name");
        assert_eq!(account.reload(app.db()).await?.unwrap().name, "new name");

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let mut conn = patch(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("name").is_some());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_token_with_access(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .with_request_json(json!({ "name": &name }))
            .run_async(&app)
            .await;

        assert_response!(conn, 202);
        let response: Account = conn.response_json().await;

        assert_eq!(&response.name, &name);
        assert_eq!(account.reload(app.db()).await?.unwrap().name, name);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_token_without_access(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let other_account = fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .with_request_json(json!({ "name": &name }))
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_admin_token(app: DivviupApi) -> TestResult {
        let account = fixtures::admin_account(&app).await;
        let other_account = fixtures::account(&app).await;
        let (_, header) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, header)
            .with_request_json(json!({ "name": &name }))
            .run_async(&app)
            .await;

        assert_response!(conn, 202);
        let response: Account = conn.response_json().await;

        assert_eq!(&response.name, &name);
        assert_eq!(other_account.reload(app.db()).await?.unwrap().name, name);
        Ok(())
    }
}
