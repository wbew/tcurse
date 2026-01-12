use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://www.recurse.com/api/v1";

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HubVisit {
    pub date: String,
    #[serde(default)]
    pub notes: Option<String>,
    pub person: VisitPerson,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VisitPerson {
    pub id: i64,
    pub name: String,
}

pub struct ApiClient {
    client: reqwest::Client,
    token: String,
}

impl ApiClient {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    pub async fn get_current_user(&self) -> Result<Profile, String> {
        let response = self.client
            .get(format!("{}/profiles/me", API_BASE))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response
            .json::<Profile>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_visit(&self, person_id: i64, date: &str) -> Result<Option<HubVisit>, String> {
        let response = self.client
            .get(format!("{}/hub_visits/{}/{}", API_BASE, person_id, date))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        let visit = response
            .json::<HubVisit>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(Some(visit))
    }

    pub async fn get_visits(&self, date: &str) -> Result<Vec<HubVisit>, String> {
        let response = self.client
            .get(format!("{}/hub_visits", API_BASE))
            .query(&[("date", date)])
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn create_or_update_visit(&self, person_id: i64, date: &str, notes: Option<&str>) -> Result<HubVisit, String> {
        let mut request = self.client
            .patch(format!("{}/hub_visits/{}/{}", API_BASE, person_id, date))
            .bearer_auth(&self.token);

        if let Some(n) = notes {
            request = request.json(&serde_json::json!({ "notes": n }));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn delete_visit(&self, person_id: i64, date: &str) -> Result<(), String> {
        let response = self.client
            .delete(format!("{}/hub_visits/{}/{}", API_BASE, person_id, date))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        Ok(())
    }
}
