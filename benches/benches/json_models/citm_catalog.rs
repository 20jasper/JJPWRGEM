use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::EmptyArray;

type Id = u32;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CitmCatalog {
    pub area_names: BTreeMap<String, String>,
    pub audience_sub_category_names: BTreeMap<String, String>,
    pub block_names: BTreeMap<String, String>,
    pub events: BTreeMap<String, Event>,
    pub performances: Vec<Performance>,
    pub seat_category_names: BTreeMap<String, String>,
    pub sub_topic_names: BTreeMap<String, String>,
    pub subject_names: BTreeMap<String, String>,
    pub topic_names: BTreeMap<String, String>,
    pub topic_sub_topics: BTreeMap<String, Vec<Id>>,
    pub venue_names: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Event {
    pub description: (),
    pub id: Id,
    pub logo: Option<String>,
    pub name: String,
    pub sub_topic_ids: Vec<Id>,
    pub subject_code: (),
    pub subtitle: (),
    pub topic_ids: Vec<Id>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Performance {
    pub event_id: Id,
    pub id: Id,
    pub logo: Option<String>,
    pub name: (),
    pub prices: Vec<Price>,
    pub seat_categories: Vec<SeatCategory>,
    pub seat_map_image: (),
    pub start: u64,
    pub venue_code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Price {
    pub amount: u32,
    pub audience_sub_category_id: Id,
    pub seat_category_id: Id,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SeatCategory {
    pub areas: Vec<Area>,
    pub seat_category_id: Id,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Area {
    pub area_id: Id,
    pub block_ids: EmptyArray,
}
