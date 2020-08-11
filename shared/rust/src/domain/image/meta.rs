use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct StyleId(pub Uuid);

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeRangeId(pub Uuid);

#[derive(Serialize, Deserialize, Debug)]
pub struct AffilitionId(pub Uuid);

#[derive(Serialize, Deserialize, Debug)]
pub struct Style {
    pub id: StyleId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeRange {
    pub id: AgeRangeId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Affiliation {
    pub id: AffilitionId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StyleResponse {
    pub styles: Vec<Style>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeRangeResponse {
    pub age_ranges: Vec<AgeRange>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AffiliationResponse {
    pub affiliations: Vec<Affiliation>,
}