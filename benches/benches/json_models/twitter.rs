use serde::{Deserialize, Serialize};

use super::EmptyArray;

type Indices = (u8, u8);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Twitter {
    pub statuses: Vec<Status>,
    pub search_metadata: SearchMetadata,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub metadata: Metadata,
    pub created_at: String,
    pub id: u64,
    pub id_str: String,
    pub text: String,
    pub source: String,
    pub truncated: bool,
    pub in_reply_to_status_id: Option<u64>,
    pub in_reply_to_status_id_str: Option<String>,
    pub in_reply_to_user_id: Option<u32>,
    pub in_reply_to_user_id_str: Option<String>,
    pub in_reply_to_screen_name: Option<String>,
    pub user: User,
    pub geo: (),
    pub coordinates: (),
    pub place: (),
    pub contributors: (),
    pub retweeted_status: Option<Box<Status>>,
    pub retweet_count: u32,
    pub favorite_count: u32,
    pub entities: StatusEntities,
    pub favorited: bool,
    pub retweeted: bool,
    pub possibly_sensitive: Option<bool>,
    pub lang: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub result_type: String,
    pub iso_language_code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub id: u32,
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
    pub location: String,
    pub description: String,
    pub url: Option<String>,
    pub entities: UserEntities,
    pub protected: bool,
    pub followers_count: u32,
    pub friends_count: u32,
    pub listed_count: u32,
    pub created_at: String,
    pub favourites_count: u32,
    pub utc_offset: Option<i32>,
    pub time_zone: Option<String>,
    pub geo_enabled: bool,
    pub verified: bool,
    pub statuses_count: u32,
    pub lang: String,
    pub contributors_enabled: bool,
    pub is_translator: bool,
    pub is_translation_enabled: bool,
    pub profile_background_color: String,
    pub profile_background_image_url: String,
    pub profile_background_image_url_https: String,
    pub profile_background_tile: bool,
    pub profile_image_url: String,
    pub profile_image_url_https: String,
    pub profile_banner_url: Option<String>,
    pub profile_link_color: String,
    pub profile_sidebar_border_color: String,
    pub profile_sidebar_fill_color: String,
    pub profile_text_color: String,
    pub profile_use_background_image: bool,
    pub default_profile: bool,
    pub default_profile_image: bool,
    pub following: bool,
    pub follow_request_sent: bool,
    pub notifications: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserEntities {
    pub url: Option<UserUrl>,
    pub description: UserUrl,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserUrl {
    pub urls: Vec<Url>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Url {
    pub url: String,
    pub expanded_url: String,
    pub display_url: String,
    pub indices: Indices,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusEntities {
    pub hashtags: Vec<Hashtag>,
    pub symbols: EmptyArray,
    pub urls: Vec<Url>,
    pub user_mentions: Vec<UserMention>,
    pub media: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hashtag {
    pub text: String,
    pub indices: Indices,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserMention {
    pub screen_name: String,
    pub name: String,
    pub id: u32,
    pub id_str: String,
    pub indices: Indices,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Media {
    pub id: u64,
    pub id_str: String,
    pub indices: Indices,
    pub media_url: String,
    pub media_url_https: String,
    pub url: String,
    pub display_url: String,
    pub expanded_url: String,
    #[serde(rename = "type")]
    pub media_type: String,
    pub sizes: Sizes,
    pub source_status_id: Option<u64>,
    pub source_status_id_str: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sizes {
    pub medium: Size,
    pub small: Size,
    pub thumb: Size,
    pub large: Size,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    pub w: u16,
    pub h: u16,
    pub resize: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchMetadata {
    pub completed_in: f64,
    pub max_id: u64,
    pub max_id_str: String,
    pub next_results: String,
    pub query: String,
    pub refresh_url: String,
    pub count: u8,
    pub since_id: u64,
    pub since_id_str: String,
}
