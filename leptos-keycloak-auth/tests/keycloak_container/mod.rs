use keycloak::{KeycloakAdmin, KeycloakAdminToken};
use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use url::Url;

/// The contained testcontainer instance implements a custom Drop function, cleaning up the running
/// container. This means that even in a panic, the container will be shut down.
#[allow(dead_code)]
pub struct KeycloakContainer {
    container: testcontainers::ContainerAsync<GenericImage>,
    pub admin_user: String,
    pub admin_password: String,
    pub port: u16,
    pub management_port: u16,
    pub url: Url,
}

impl KeycloakContainer {
    pub async fn start() -> Self {
        tracing::info!("Starting Keycloak...");

        let admin_user = "admin".to_owned();
        let admin_password = "admin".to_owned();

        // This setup is roughly equivalent to the following cli command:
        // `docker run -p 8080:8080 -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:25.0.0 start-dev`

        let keycloak_image = GenericImage::new("quay.io/keycloak/keycloak", "25.0.0")
            .with_exposed_port(ContainerPort::Tcp(8080))
            .with_wait_for(WaitFor::message_on_stdout(
                "Keycloak 25.0.0 on JVM (powered by Quarkus 3.8.5) started",
            ))
            .with_wait_for(WaitFor::message_on_stdout(
                "Listening on: http://0.0.0.0:8080",
            ))
            .with_wait_for(WaitFor::message_on_stdout(
                "Management interface listening on http://0.0.0.0:9000",
            ));

        let container_request = keycloak_image
            .with_env_var("KEYCLOAK_ADMIN", admin_user.as_str())
            .with_env_var("KEYCLOAK_ADMIN_PASSWORD", admin_password.as_str())
            .with_env_var("KC_HTTP_ENABLED", "true")
            .with_env_var("KC_HOSTNAME_STRICT_HTTPS", "false")
            .with_cmd(["start-dev"]);

        let container = container_request.start().await.expect("Keycloak started");

        let port = container
            .get_host_port_ipv4(8080)
            .await
            .expect("Keycloak to export port 8080");

        let management_port = container
            .get_host_port_ipv4(8080)
            .await
            .expect("Keycloak to export port 9000");

        let url = Url::parse(format!("http://127.0.0.1:{}", port).as_str()).unwrap();
        tracing::info!(available_at = ?url, "Keycloak started.");

        Self {
            container,
            admin_user,
            admin_password,
            port,
            management_port,
            url,
        }
    }

    pub async fn admin_client(&self) -> KeycloakAdmin {
        let client = reqwest::Client::new();
        let admin_token = KeycloakAdminToken::acquire(
            self.url.as_str(),
            &self.admin_user,
            &self.admin_password,
            &client,
        )
        .await
        .expect("Correct credentials");

        KeycloakAdmin::new(self.url.as_str(), admin_token, client)
    }
}
