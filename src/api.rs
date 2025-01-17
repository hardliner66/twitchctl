use std::error::Error;
use twitch_api2::{
    helix::{
        search::{search_categories::Category, SearchCategoriesRequest},
        streams::get_stream_tags::GetStreamTagsRequest,
        tags::{GetAllStreamTagsRequest, TwitchTag},
        users::GetUsersRequest,
    },
    twitch_oauth2::{AccessToken, TwitchToken, UserToken},
    HelixClient,
};
use twitch_oauth2::client::surf_http_client;

use derivative::Derivative;

async fn get_user(token_string: String) -> Result<UserToken, Box<dyn Error + 'static>> {
    let token =
        UserToken::from_existing(surf_http_client, AccessToken::new(token_string), None, None)
            .await?;
    token.validate_token(surf_http_client).await?;

    Ok(token)
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ApiClient<'a> {
    #[derivative(Debug = "ignore")]
    helix_client: HelixClient<'a, surf::Client>,
    token: UserToken,
}

impl<'a> ApiClient<'a> {
    pub async fn new(token: String) -> Result<ApiClient<'a>, Box<dyn Error>> {
        Ok(ApiClient {
            helix_client: HelixClient::with_client(surf::Client::new()),
            token: get_user(token).await?,
        })
    }

    pub async fn search_categories(
        &self,
        term: String,
        max: Option<usize>,
    ) -> Result<Option<Vec<Category>>, Box<dyn Error>> {
        // TODO Implement some better filter (only starting with for example) to reduce the number
        // of results for searches

        // TODO Maybe only return Some(Category) for one result.

        // FIXME This can throw an error when noting is found https://github.com/Emilgardis/twitch_api2/issues/92

        let req = SearchCategoriesRequest::builder()
            .query(term)
            .first(max.unwrap_or(20).max(1).min(100).to_string())
            .build();
        let res: Vec<Category> = self.helix_client.req_get(req, &self.token).await?.data;
        if res.len() > 0 {
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
    pub async fn get_stream_tags(&self, login: String) -> Result<Vec<TwitchTag>, Box<dyn Error>> {
        let channel_info_req = GetUsersRequest::builder().login(vec![login]).build();
        let channel_info_res = self
            .helix_client
            .req_get(channel_info_req, &self.token)
            .await?;
        let tag_req = GetStreamTagsRequest::builder()
            .broadcaster_id(channel_info_res.data[0].id.to_string())
            .build();
        let tag_res = self.helix_client.req_get(tag_req, &self.token).await?;
        Ok(tag_res.data)
    }
    pub async fn get_all_tags(&self) -> Result<Vec<TwitchTag>, Box<dyn Error>> {
        let mut tags = vec![];
        let mut pagination = None;
        loop {
            let req = GetAllStreamTagsRequest::builder()
                .after(pagination)
                .first(Some(100))
                .build();
            let mut res = self.helix_client.req_get(req, &self.token).await?;
            tags.append(&mut res.data);
            pagination = res.pagination;
            if pagination == None {
                break;
            }
        }
        Ok(tags)
    }
}
