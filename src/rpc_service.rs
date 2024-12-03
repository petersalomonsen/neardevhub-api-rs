use devhub_shared::proposal::VersionedProposal;
use devhub_shared::rfp::VersionedRFP;
use near_account_id::AccountId;
use near_api::{prelude::*, types::reference::Reference, types::Data};
use near_jsonrpc_client::methods::query::RpcQueryRequest;
use rocket::http::Status;
use rocket::serde::json::json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RpcResponse {
    pub data: String,
}

#[derive(Clone)]
pub struct RpcService {
    network: NetworkConfig,
    contract: Contract,
}

#[derive(Deserialize)]
struct QueryResponse {
    // jsonrpc: String,
    result: QueryResponseResult,
    // id: String,
}

#[derive(Deserialize)]
struct QueryResponseResult {
    // result is an array of bytes, to be specific it is an ASCII code of the string
    result: Vec<i32>,
    // block_hash: String,
    // block_height: i64,
    // logs: Vec<String>,
}

impl Default for RpcService {
    fn default() -> Self {
        Self {
            network: NetworkConfig::mainnet(),
            contract: Contract("devhub.near".parse::<AccountId>().unwrap()),
        }
    }
}

impl RpcService {
    pub fn new(id: AccountId) -> Self {
        Self {
            network: NetworkConfig::mainnet(),
            contract: Contract(id),
        }
    }

    pub async fn get_proposal(
        &self,
        proposal_id: i32,
    ) -> Result<Data<VersionedProposal>, near_api::errors::QueryError<RpcQueryRequest>> {
        let result: Result<Data<VersionedProposal>, _> = self
            .contract
            .call_function("get_proposal", json!({ "proposal_id": proposal_id }))
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        result
    }

    pub async fn get_rfp(
        &self,
        rfp_id: i32,
    ) -> Result<Data<VersionedRFP>, near_api::errors::QueryError<RpcQueryRequest>> {
        let result: Result<Data<VersionedRFP>, _> = self
            .contract
            .call_function("get_rfp", json!({ "rfp_id": rfp_id }))
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        result
    }

    pub async fn get_all_proposal_ids(&self) -> Result<Vec<i32>, Status> {
        let result: Result<Data<Vec<i32>>, _> = self
            .contract
            .call_function("get_all_proposal_ids", ())
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        match result {
            Ok(res) => Ok(res.data),
            Err(e) => {
                eprintln!("Failed to get all proposal ids: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    }

    pub async fn get_proposal_on_block(
        &self,
        proposal_id: i32,
        block_id: i64,
    ) -> Result<VersionedProposal, Status> {
        let result: Result<Data<VersionedProposal>, near_api::errors::QueryError<RpcQueryRequest>> =
            self.contract
                .call_function("get_proposal", json!({ "proposal_id": proposal_id }))
                .unwrap()
                .read_only()
                .at(Reference::AtBlock(block_id as u64))
                .fetch_from(&self.network)
                .await;

        match result {
            Ok(res) => Ok(res.data),
            Err(e) => {
                // After Block 122723375 & 122938305
                // TODO caught error:
                // Failed to get proposal on block: DeserializeError(Error("missing field `kyc_verified`", line: 0, column: 0))
                eprintln!(
                    "Failed to get proposal on block: {:?}, Error: {:?}",
                    block_id, e
                );
                Err(Status::InternalServerError)
            }
        }
    }

    pub async fn get_rfp_on_block(
        &self,
        rfp_id: i32,
        block_id: i64,
    ) -> Result<VersionedRFP, Status> {
        let result: Result<Data<VersionedRFP>, near_api::errors::QueryError<RpcQueryRequest>> =
            self.contract
                .call_function("get_rfp", json!({ "rfp_id": rfp_id }))
                .unwrap()
                .read_only()
                .at(Reference::AtBlock(block_id as u64))
                .fetch_from(&self.network)
                .await;

        match result {
            Ok(res) => Ok(res.data),
            Err(e) => {
                eprintln!("Failed to get rfp on block: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    }

    pub async fn query(
        &self,
        method_name: String,
        block_id: i64,
        args_base64: String,
    ) -> Result<String, Status> {
        let args = json!({
          "request_type": "call_function",
          "account_id": self.contract.0.to_string(),
          "block_id": block_id,
          "method_name": method_name,
          "args_base64": args_base64
        });

        println!("Querying args: {:?}", args);

        let result: Result<Data<QueryResponse>, _> = self
            .contract
            .call_function("query", args)
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        match result {
            Ok(res) => {
                // From ascii code to string
                let decoded = res
                    .data
                    .result
                    .result
                    .iter()
                    .map(|c| *c as u8 as char)
                    .collect();
                // Should return JSON object?
                Ok(decoded)
            }
            Err(e) => {
                eprintln!("Failed to query: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    }
}
