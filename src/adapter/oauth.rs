use oauth2::basic::BasicClient;
use oauth2::{ClientId, ClientSecret};

pub struct OauthClient {
    oauth: BasicClient,
}

impl OauthClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        OauthClient {
            oauth: BasicClient::new(ClientId::new(client_id))
                .set_client_secret(ClientSecret::new(client_secret))
                .
        }
    }
}