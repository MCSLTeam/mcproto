# mcproto-oauth

Microsoft OAuth login for Minecraft. The crate implements authorization-code
redirect login with PKCE and device-code login, then exchanges the resulting
token through Xbox Live, XSTS, and Minecraft Services.

The application owns its Microsoft client ID. This crate does not provide or
embed a default client ID.

The implementation is divided into three public modules:

- `redirect_uri`: PKCE authorization URL, local callback, and Microsoft token.
- `device_code`: device authorization, polling, and Microsoft token refresh.
- `xbox_login`: shared Xbox Live, XSTS, and Minecraft Services exchange.

The top-level types are asynchronous. Equivalent blocking types are available
under `mcproto_oauth::sync` and do not require a Tokio runtime.

```rust,no_run
use mcproto_oauth::RedirectFlow;

# async fn example() -> Result<(), mcproto_oauth::Error> {
let client_id = std::env::var("MICROSOFT_CLIENT_ID")
    .expect("MICROSOFT_CLIENT_ID must be set");
let flow = RedirectFlow::new(client_id, "http://localhost:yourport")?;
let session = flow.start().await?;

println!("Open {}", session.authorization_url());
let login = session.complete().await?;
println!("Minecraft user: {}", login.minecraft.username);
# Ok(())
# }
```

Each stage can also be inspected and executed separately:

```rust,no_run
use mcproto_oauth::{RedirectFlow, XboxLogin};

# async fn example() -> Result<(), mcproto_oauth::Error> {
# let client_id = "application-client-id";
let flow = RedirectFlow::new(client_id, "http://localhost:yourport")?;
let session = flow.start().await?;

println!("{}", session.authorization_url());
println!("{}", session.code_challenge());
println!("{}", session.code_verifier().expose());
println!("{}", session.state().expose());

let authorization = session.receive_code().await?;
println!("{}", authorization.code.expose());

let microsoft = flow.exchange_code(&authorization).await?;
let xbox = XboxLogin::new()?;
let xbox_live = xbox.authenticate_xbox_live(&microsoft.access_token).await?;
let xsts = xbox.authorize_xsts(&xbox_live).await?;
let identity_token = xsts.minecraft_identity_token();
let minecraft = xbox.authenticate_minecraft(&xsts).await?;
# let _ = (identity_token, minecraft);
# Ok(())
# }
```

Device-code login can also run in one call after displaying the instructions:

```rust,no_run
use mcproto_oauth::DeviceCodeFlow;

# async fn example() -> Result<(), mcproto_oauth::Error> {
# let client_id = "application-client-id";
let flow = DeviceCodeFlow::new(client_id)?;
let session = flow.start().await?;

println!("Open {}", session.verification_uri);
println!("Enter {}", session.user_code);

let login = session.complete().await?;
println!("Minecraft user: {}", login.minecraft.username);
# Ok(())
# }
```

Or the application can control every poll and downstream exchange:

```rust,no_run
use mcproto_oauth::{DeviceCodeFlow, DeviceCodePoll, XboxLogin};

# async fn example() -> Result<(), mcproto_oauth::Error> {
# let client_id = "application-client-id";
let flow = DeviceCodeFlow::new(client_id)?;
let mut session = flow.start().await?;

let microsoft = loop {
    tokio::time::sleep(session.poll_interval()).await;
    match session.poll_once().await? {
        DeviceCodePoll::Complete(token) => break token,
        DeviceCodePoll::AuthorizationPending { retry_after, .. }
        | DeviceCodePoll::SlowDown { retry_after, .. } => {
            println!("Retry after {retry_after:?}");
        }
    }
};

let xbox = XboxLogin::new()?;
let xbox_live = xbox.authenticate_xbox_live(&microsoft.access_token).await?;
let xsts = xbox.authorize_xsts(&xbox_live).await?;
let minecraft = xbox.authenticate_minecraft(&xsts).await?;
# let _ = minecraft;
# Ok(())
# }
```

The synchronous API has the same complete and step-by-step structure:

```rust,no_run
use mcproto_oauth::sync::DeviceCodeFlow;

# fn example() -> Result<(), mcproto_oauth::Error> {
# let client_id = "application-client-id";
let flow = DeviceCodeFlow::new(client_id)?;
let session = flow.start()?;

println!("Open {}", session.verification_uri);
println!("Enter {}", session.user_code);

let login = session.complete()?;
println!("Minecraft user: {}", login.minecraft.username);
# Ok(())
# }
```

For synchronous redirect login, use `mcproto_oauth::sync::RedirectFlow`; for
individual blocking Xbox stages, use `mcproto_oauth::sync::XboxLogin`.

The redirect URI must be registered for the client ID and must use HTTP on
`localhost` or a loopback IP address. The listener binds before the
authorization URL is returned, so a fast browser redirect cannot race it.

## License

Licensed under the [MIT License](../LICENSE).
