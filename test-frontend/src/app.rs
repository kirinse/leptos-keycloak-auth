use std::ops::Deref;

use crate::routes::routes;
use gloo_utils::format::JsValueSerdeExt;
use leptos::ev::message;
use leptos::html::Iframe;
use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::tachys::renderer::dom::Event;
use leptos_keycloak_auth::components::{DebugState, ShowWhenAuthenticated};
use leptos_keycloak_auth::url::Url;
use leptos_keycloak_auth::{
    UseKeycloakAuthOptions, ValidationOptions, expect_authenticated, expect_keycloak_auth,
    init_keycloak_auth, to_current_url,
};
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::*;
use leptos_use::{UseTimeoutFnReturn, use_event_listener, use_timeout_fn, use_window};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::HtmlIFrameElement;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", content = "data", rename_all = "camelCase")]
pub enum RcvMessage {
    Close,
    Reload,
    RegisterContentDimensions {},
    FirstIframeLoad {
        origin: String,
    },
    #[serde(rename_all = "camelCase")]
    ContentLoaded {
        content_type: String,
    },
    #[serde(rename_all = "camelCase")]
    FormSubmission {
        form_type: String,
    },
    InputFocused {},
    InputBlured,
    BeforeIframeLocationChange {
        event: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", content = "data", rename_all = "camelCase")]
pub enum SndMessage {
    RegisterOrigin {
        origin: String,
    },
    ContainerIsReady,
    ViewportResize,
    IframeResize,
    EnvironmentResize,
    FocusOnInput,
    #[serde(rename_all = "camelCase")]
    LoadContent {
        content_type: String,
    },
    ProceedWithLocationChange {
        event: String,
    },
    UpdateInputField,
    SelectField,
    ClickOnField,
    GamepadBackButtonPressed,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Meta name="charset" content="UTF-8"/>
        <Meta name="description" content="Leptonic SSR template"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="theme-color" content="#8856e6"/>

        <Stylesheet id="leptos" href="/pkg/frontend.css"/>
        <Stylesheet href="https://fonts.googleapis.com/css?family=Roboto&display=swap"/>

        <Title text="Leptonic SSR template"/>

        // <Root default_theme=LeptonicTheme::default()>
            <main style=r#"
                height: 100%;
                width: 100%;
                display: flex;
                flex-direction: column;
                align-items: center;
                padding: 1em;
                background-color: antiquewhite;
                overflow: auto;
            "#>
                <Router>
                    { routes::generated_routes() }
                </Router>
            </main>
            <div class="GalaxyAccountsFrameContainer__measure"></div>
        // </Root>
    }
}

#[component]
pub fn MainLayout() -> impl IntoView {
    view! {
        <Outlet />
    }
}

#[component]
pub fn Welcome() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <h2>Welcome to Leptonic</h2>

        <a id="to-public" href=routes::root::Public.materialize().strip_prefix("/").unwrap().to_string()>
            Public area
        </a>

        <a id="to-my-account" href=routes::root::MyAccount.materialize()>
            "My Account"
        </a>

        <span id="count" style="margin-top: 3em;">
            "Count: " { move || count.get() }
        </span>

        <button id="increase" on:click:target=move|_| set_count.update(|c| *c += 1)>
            Increase
        </button>
    }
}

#[component]
pub fn Public() -> impl IntoView {
    view! {
        <h2>"Welcome to Leptonic"</h2>

        <a id="to-my-account" href=routes::Root.materialize()>
            Back
        </a>
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct WhoAmIResponse {
    username: String,
    keycloak_uuid: String,
    token_valid_for_whole_seconds: i32,
}

#[component]
pub fn MyAccount() -> impl IntoView {
    let auth = expect_keycloak_auth();
    let authenticated = expect_authenticated();

    let auth_state = Signal::derive(move || auth.state_pretty_printer());
    let user_name = Signal::derive(move || authenticated.id_token_claims.read().name.clone());
    let logout_url = Signal::derive(move || auth.logout_url.get().map(|url| url.to_string()));
    let logout_url_unavailable = Signal::derive(move || logout_url.get().is_none());

    // let who_am_i = LocalResource::new(move || {
    //     let client = authenticated.client();
    //     async move {
    //         client
    //             .get("http://127.0.0.1:9999/who-am-i")
    //             .await
    //             .unwrap()
    //             .json::<WhoAmIResponse>()
    //             .await
    //             .unwrap()
    //     }
    // });

    view! {
        <h1 id="greeting">
            "Hello, " { move || user_name.get() } "!"
        </h1>

        // <Suspense fallback=|| view! { "" }>
        //     { move || who_am_i.get().map(|who_am_i| view! {
        //         <div>"username: " <span id="username">{ who_am_i.username.clone() }</span></div>
        //         <div>"keycloak_uuid: " <span id="keycloak_uuid">{ who_am_i.keycloak_uuid.clone() }</span></div>
        //         <div>"token_valid_for_whole_seconds: " <span id="token_valid_for_whole_seconds">{ who_am_i.token_valid_for_whole_seconds }</span></div>
        //     }) }
        // </Suspense>

        <pre id="auth-state" style="width: 100%; overflow: auto;">
            { move || auth_state.read()() }
        </pre>
        <a id="logout" href=move || logout_url.get().unwrap_or_default() aria_disabled=logout_url_unavailable>Logout</a>
        <button id="programmatic-logout" on:click:target=move |_| auth.end_session()>Programmatic Logout</button>
        <a id="back-to-root" href=routes::Root.materialize() style:margin-top="3em">Back to root</a>
    }
}

#[server]
async fn get_keycloak_port() -> Result<u16, ServerFnError> {
    let _ = dotenvy::dotenv().ok();
    let port = std::env::var("KEYCLOAK_PORT").expect("KEYCLOAK_PORT must be set");
    let keycloak_port = port.parse::<u16>().expect("KEYCLOAK_PORT to be a u16");
    tracing::info!(keycloak_port, "parsed KEYCLOAK_PORT");
    Ok(keycloak_port)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeycloakPort(u16);

#[component]
pub fn Protected(children: ChildrenFn) -> impl IntoView {
    // Note: Use a `LocalResource` with a `Suspend` to force rendering of the protected are
    // client-side only. We should also not execute `use_keycloak_auth` on the server, as it has
    // no support for SSR yet.
    //
    // Our test-setup starts Keycloak with randomized ports, so we cannot hardcode "8443" here,
    // but can actually make use of the enforced resource to asynchronously retrieve the port.
    let keycloak_port = LocalResource::new(|| async move { get_keycloak_port().await.unwrap() });

    view! {
        <Suspense fallback=|| view! { "" }>
            {Suspend::new(async move {
                let port = keycloak_port.await;
                provide_context(KeycloakPort(port));
                tracing::info!(port, "Initializing Keycloak auth...");
                let keycloak_server_url = format!("http://localhost:{port}");
                let _auth = init_keycloak_auth(UseKeycloakAuthOptions {
                    keycloak_server_url: Url::parse(&keycloak_server_url).unwrap(),
                    realm: "myrealm".to_owned(),
                    client_id: "myclient".to_owned(),
                    post_login_redirect_url: to_current_url(),
                    post_logout_redirect_url: to_current_url(),
                    scope: vec![],
                    id_token_validation: ValidationOptions {
                        expected_audiences: Some(vec!["myclient".to_owned()]),
                        expected_issuers: Some(vec![format!("{keycloak_server_url}/realms/myrealm")]),
                    },
                    delay_during_hydration: false,
                    advanced: Default::default(),
                });
                view! {
                    <ShowWhenAuthenticated fallback=|| view! { <Login/> }>
                        { children() }
                    </ShowWhenAuthenticated>

                    <DebugState/>
                }
            })}
        </Suspense>
    }
}

fn get_origin() -> String {
    if let Some(window) = use_window().deref() {
        let origin = window.location().origin().unwrap_or_else(|_| {
            let location = window.location();
            format!(
                "{}//{}{}",
                location.protocol().unwrap_or_default(),
                location.hostname().unwrap_or_default(),
                location.port().unwrap_or_default()
            )
        });
        return origin;
    }
    "*".into()
}

#[component]
pub fn Login() -> impl IntoView {
    let auth = expect_keycloak_auth();
    let login_url_unavailable = Signal::derive(move || auth.login_url.get().is_none());
    let login_url = Signal::derive(move || {
        auth.login_url
            .get()
            .map(|url| url.to_string())
            .unwrap_or_default()
    });
    let keycloak_port = expect_context::<KeycloakPort>();
    let auth_state = Signal::derive(move || auth.state_pretty_printer());

    let (openned, set_openned) = signal(false);
    let (loaded, set_loaded) = signal(false);

    let mut origin = "*".to_string();

    let frame_ref = NodeRef::<Iframe>::new();
    let frame_window = move || {
        frame_ref
            .get()
            .map(|f| f.content_window().unwrap())
            .unwrap()
    };

    let UseTimeoutFnReturn { start: _, .. } = use_timeout_fn(
        |node: HtmlIFrameElement| {
            if let Some(w) = node.content_window() {
                let origin = window().location().origin().unwrap_or_else(|_| {
                    format!(
                        "{}//{}{}",
                        window().location().protocol().unwrap_or_default(),
                        window().location().hostname().unwrap_or_default(),
                        window().location().port().unwrap_or_default()
                    )
                });
                let res = w.post_message(
                    JsValue::from_serde(&SndMessage::RegisterOrigin { origin })
                        .unwrap()
                        .as_ref(),
                    "*",
                );
                leptos::logging::log!("{:?}", res);
            }
        },
        1000.0,
    );

    Effect::new(move |_| {
        if !openned.get() {
            set_loaded.set(false);
        }
    });

    let _cleanup = use_event_listener(window(), message, move |evt| {
        let src = evt.source();
        tracing::debug!("{:?}", &evt.data());
        // let f = frame_ref.get().map(|f| f.content_window().unwrap());
        // logging::log!("src = {:?}, frame = {:?}, {}", &src, &f, src == f,);
        // check event source
        if src != Some(frame_window().into()) {
            return;
        }
        let data = evt.data().into_serde::<RcvMessage>();
        tracing::debug!(?data);
        if let Ok(msg) = data {
            match msg {
                RcvMessage::Close => set_openned.set(false),
                RcvMessage::Reload => window().location().reload().unwrap_or_default(),
                RcvMessage::RegisterContentDimensions {} => {
                    set_loaded.set(true);
                    let _ = frame_window().post_message(
                        JsValue::from_serde(&SndMessage::ContainerIsReady)
                            .unwrap()
                            .as_ref(),
                        &origin,
                    );
                }
                RcvMessage::BeforeIframeLocationChange { event } => {
                    set_loaded.set(false);
                    let _ = frame_window().post_message(
                        JsValue::from_serde(&SndMessage::ProceedWithLocationChange { event })
                            .unwrap()
                            .as_ref(),
                        &origin,
                    );
                }
                RcvMessage::FirstIframeLoad { origin: ori } => {
                    origin = ori;
                    let _ = frame_window().post_message(
                        JsValue::from_serde(&SndMessage::RegisterOrigin {
                            origin: get_origin(),
                        })
                        .unwrap()
                        .as_ref(),
                        &origin,
                    );
                    let _ = frame_window().post_message(
                        JsValue::from_serde(&SndMessage::LoadContent {
                            content_type: String::new(),
                        })
                        .unwrap()
                        .as_ref(),
                        &origin,
                    );
                }
                RcvMessage::ContentLoaded { content_type: _ } => {
                    set_loaded.set(true);
                }
                _ => {}
            }
        }
    });

    view! {
       <h1 id="unauthenticated">"Unauthenticated"</h1>

        <div id="keycloak-port">
            { keycloak_port.0 }
        </div>

        <pre id="auth-state" style="width: 100%; overflow: auto;">
            { move || auth_state.read()() }
        </pre>

        <button
            disabled=login_url_unavailable
            on:click:target={move |_| {
                tracing::debug!("clicked");
                set_openned.set(!openned.get());
            }}
        >
            Log in
        </button>

        <a id="back-to-root" href=routes::Root.materialize() style="margin-top: 3em;">
            Back to root
        </a>

        {move || openned.with(|open| {
            if *open {
                view!{
                    <Portal mount={document().body().unwrap()}>
                        <div class="GalaxyAccountsFrameContainer__overlay"></div>
                        <div id="GalaxyAccountsFrameContainer" class=("l-loaded", loaded)>
                            <div class="GalaxyAccountsFrameContainer__container">
                                <iframe node_ref={frame_ref} id="GalaxyAccountsFrame" name="galaxyAccounts" src={move || login_url.get()}>
                                </iframe>
                            </div>
                        </div>
                    </Portal>
                }.into_any()
            } else {
                view!{<></>}.into_any()
            }
        })}
    }
}
