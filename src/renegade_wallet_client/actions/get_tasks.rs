//! Gets the wallet's task history from the historical state engine

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        request_response::{GetTasksQueryParameters, GetTasksResponse},
        tasks::ApiTask,
        GET_TASKS_ROUTE,
    },
    RenegadeClientError,
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
        let mut query_params = GetTasksQueryParameters {
            include_historic_tasks: Some(include_historic_tasks),
            ..Default::default()
        };
        let path = self.build_get_tasks_request_path(&query_params)?;

        let GetTasksResponse { mut tasks, mut next_page_token } =
            self.relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            query_params.page_token = Some(page_token);
            let path = self.build_get_tasks_request_path(&query_params)?;

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
        query_params: &GetTasksQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(GET_TASKS_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
