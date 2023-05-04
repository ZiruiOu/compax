use compax::{paxos_kv, BallotID};
use paxos_kv::paxos_service_client::PaxosServiceClient;
use paxos_kv::{BallotId, Proposal, ProposeRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "http://[::1]:50051";
    let mut client_stub = PaxosServiceClient::connect(address).await?;

    let ballot_id = BallotID::new(1, 2);

    let proposal = Proposal {
        propose_id: Some(ballot_id.into()),
        propose_value: 114514,
    };

    let request = Request::new(ProposeRequest {
        proposal: proposal.into(),
    });

    let reply = client_stub.propose(request).await?;

    println!("Get reply: {:?}", reply);

    Ok(())
}
