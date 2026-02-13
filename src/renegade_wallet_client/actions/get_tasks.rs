//! Gets the wallet's task history from the historical state engine

use renegade_external_api::{
    http::task::{GET_TASKS_ROUTE, GetTasksResponse},
    types::ApiTask,
};

use crate::{
    RenegadeClientError,
    actions::{INCLUDE_HISTORIC_TASKS_PARAM, PAGE_TOKEN_PARAM, construct_http_path},
    client::RenegadeClient,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Fetches all tasks in the account, optionally including historic tasks.
    ///
    /// This method will paginate through all of the account's tasks across
    /// multiple requests, returning them all.
    pub async fn get_tasks(
        &self,
        include_historic_tasks: bool,
    ) -> Result<Vec<ApiTask>, RenegadeClientError> {
        let path = self.build_get_tasks_request_path(include_historic_tasks, None)?;

        let GetTasksResponse { mut tasks, mut next_page_token } =
            self.relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            let path =
                self.build_get_tasks_request_path(include_historic_tasks, Some(page_token))?;

            let response: GetTasksResponse = self.relayer_client.get(&path).await?;

            tasks.extend(response.tasks);
            next_page_token = response.next_page_token;
        }

        Ok(tasks)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request path for the get tasks endpoint
    fn build_get_tasks_request_path(
        &self,
        include_historic_tasks: bool,
        page_token: Option<i64>,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(GET_TASKS_ROUTE, "account_id" => self.get_account_id());

        let mut params = vec![(INCLUDE_HISTORIC_TASKS_PARAM, include_historic_tasks.to_string())];
        if let Some(token) = page_token {
            params.push((PAGE_TOKEN_PARAM, token.to_string()));
        }

        let query_string =
            serde_urlencoded::to_string(&params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
