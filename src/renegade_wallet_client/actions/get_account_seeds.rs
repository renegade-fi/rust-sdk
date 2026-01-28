//! Get an account's seed CSPRNG states from the relayer

use renegade_darkpool_types::csprng::PoseidonCSPRNG;

use crate::{
    RenegadeClientError,
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{GET_ACCOUNT_SEEDS_ROUTE, request_response::GetAccountSeedsResponse},
};

impl RenegadeClient {
    /// Get an account's seed CSPRNG states from the relayer.
    /// These are the CSPRNGs used to sample seeds with which to create new
    /// state objects.
    ///
    /// Returns a tuple of (recovery stream seeds CSPRNG, share stream seeds
    /// CSPRNG)
    // TODO: Store the CSPRNGs in the client behind a Mutex
    pub async fn get_account_seeds(
        &self,
    ) -> Result<(PoseidonCSPRNG, PoseidonCSPRNG), RenegadeClientError> {
        let path =
            construct_http_path!(GET_ACCOUNT_SEEDS_ROUTE, "account_id" => self.get_account_id());

        let GetAccountSeedsResponse { recovery_seed_csprng, share_seed_csprng } =
            self.relayer_client.get(&path).await?;

        Ok((recovery_seed_csprng.into(), share_seed_csprng.into()))
    }
}
