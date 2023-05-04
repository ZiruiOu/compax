use std::cmp::{Ord, Ordering};
use std::convert::{From, Into};

pub mod paxos_kv {
    tonic::include_proto!("paxoskv");
}

#[derive(Debug, Clone, Eq, PartialOrd)]
pub struct BallotID {
    propose_number: u64,
    proposer_id: u64,
}

impl BallotID {
    pub fn new(propose_number: u64, proposer_id: u64) -> Self {
        BallotID {
            propose_number: propose_number,
            proposer_id: proposer_id,
        }
    }
}

impl PartialEq for BallotID {
    fn eq(&self, other: &Self) -> bool {
        self.propose_number == other.propose_number && self.proposer_id == other.proposer_id
    }
}

impl Ord for BallotID {
    fn cmp(&self, other: &BallotID) -> Ordering {
        let result = self.propose_number.cmp(&other.propose_number);
        match result {
            Ordering::Equal => self.proposer_id.cmp(&other.proposer_id),
            _ => result,
        }
    }
}

impl From<paxos_kv::BallotId> for BallotID {
    fn from(ballot_id: paxos_kv::BallotId) -> Self {
        BallotID {
            propose_number: ballot_id.propose_number,
            proposer_id: ballot_id.proposer_id,
        }
    }
}

impl From<BallotID> for paxos_kv::BallotId {
    fn from(ballot_id: BallotID) -> Self {
        paxos_kv::BallotId {
            propose_number: ballot_id.propose_number,
            proposer_id: ballot_id.proposer_id,
        }
    }
}

//impl Into<paxos_kv::BallotId> for BallotID {
//    fn into(self) -> paxos_kv::BallotId {
//        paxos_kv::BallotId {
//            propose_number: self.propose_number,
//            proposer_id: self.proposer_id,
//        }
//    }
//}
