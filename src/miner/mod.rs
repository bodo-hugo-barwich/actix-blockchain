/*
* @author Bodo (Hugo) Barwich
* @version 2024-11-30
* @package Blockchain Exercise
* @subpackage Blockchain Miner Actor

* This Module defines the Actor that spawns dedicated Blockchain Mining Thread
*
*---------------------------------
* Requirements:
*/

use actix::prelude::*;
use actix::Addr;
use actix_web::web;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

use rand::distributions::{Distribution, Uniform};
//use tokio::time::{sleep, Duration};
use std::{thread, time};

use crate::model::blockchain::Blockchain;

//==============================================================================
// Structure MiningMessage Declaration

/// Structure for Managing the Mining Process
#[derive(Debug, Message)]
#[rtype(result = "Result<MiningResponse, MiningError>")]
pub struct MiningMessage;

/// Structure for Email Sending Results
#[derive(Debug, Serialize, Deserialize)]
pub struct MiningResponse {
    pub status: String,
    pub report: String,
}

/// Structure for Email Sending Errors
#[derive(Debug, Serialize, Deserialize)]
pub struct MiningError {
    status: String,
    report: String,
}

//==============================================================================
// Structure EmailSender Declaration

/// Structure for executing the Mining Process
// Define actor
pub struct MiningWorker {
    blockchain_mutex: web::Data<Mutex<Blockchain>>,
}

//==============================================================================
// Structure MiningWorker Implementation

impl Default for MiningWorker {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        Self::new()
    }
}

impl MiningWorker {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> Self {
        Self {
            blockchain_mutex: web::Data::new(Mutex::new(Blockchain::new())),
        }
    }

    pub fn with_data(data_mutex: web::Data<Mutex<Blockchain>>) -> Self {
        Self {
            blockchain_mutex: data_mutex,
        }
    }

    /*----------------------------------------------------------------------------
     * Administration Methods
     */

    pub fn set_data(&mut self, data_mutex: web::Data<Mutex<Blockchain>>) {
        self.blockchain_mutex = data_mutex;
    }

    pub fn mine_block(&mut self) -> Result<u64, MiningError> {
        match self.blockchain_mutex.lock() {
            Ok(mut guard) => {
                println!("Start Mining ...");

                let blockchain = guard.deref_mut();

                let proof = blockchain.proof_of_work();

                let _ = blockchain.add_transaction_from_data("blockchain", "Miner", 10f64);

                Ok(proof)
            }
            Err(e) => Err(MiningError {
                status: "failed".to_owned(),
                report: format!("Blockchain: Mutex Lock failed! Message: {:?}", e),
            }),
        }
    }

    /*----------------------------------------------------------------------------
     * Consultation Methods
     */

    pub fn get_block_count(&self) -> usize {
        match self.blockchain_mutex.lock() {
            Ok(guard) => {
                let blockchain = guard.deref();

                blockchain.chain.len()
            }
            Err(e) => {
                eprintln!("Blockchain: Mutex Lock failed! Message: {:?}", e);
                0
            }
        }
    }
}

// Provide Actor implementation for EmailSender
impl Actor for MiningWorker {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Mining Worker Actor is alive");

        let mut rng = rand::thread_rng();
        let sleep_range = Uniform::from(1..20);

        let sleep_msecs = sleep_range.sample(&mut rng);

        println!("random delay: {} ms", sleep_msecs);

        thread::sleep(time::Duration::from_millis(sleep_msecs));

        println!("random delay: delay complete");

        let block_count = self.get_block_count();

        if block_count == 0 {
            // Generate Genesis Block
            match self.mine_block() {
                Ok(proof) => {
                    println!("Block (Index: 1; Proof: {}): Genesis Block mined", proof);
                }
                Err(e) => {
                    eprintln!("Block (Index: 1): Block Mining failed: {:?}", e);
                }
            }
        }

        println!("blockchain state: mutex aquiring ...");

        println!("blockchain mutex: {:?}", self.blockchain_mutex);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Mining Worker Actor is stopped");
    }
}

/// Define handler for `MiningMessage` message
impl Handler<MiningMessage> for MiningWorker {
    type Result = Result<MiningResponse, MiningError>;

    fn handle(&mut self, _msg: MiningMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Mine a new Block
        let proof = self.mine_block()?;
        let block_index;
        let block_json;

        match self.blockchain_mutex.lock() {
            Ok(guard) => {
                let blockchain = guard.deref();

                let block = blockchain.get_last_block();

                match block {
                    Some(b) => {
                        block_index = b.index;
                        block_json = match b.to_json() {
                            Ok(j) => j,
                            Err(e) => format!(
                                "Block ({}): JSON formatting failed! Message: {:?}",
                                block_index, e
                            ),
                        };
                    }
                    None => {
                        return Err(MiningError {
                            status: "empty".to_owned(),
                            report: "Blockchain: Blockchain is empty".to_owned(),
                        });
                    }
                }
            }
            Err(e) => {
                return Err(MiningError {
                    status: "failed".to_owned(),
                    report: format!("Blockchain: Mutex Lock failed! Message: {:?}", e),
                });
            }
        }

        Ok(MiningResponse {
            status: "success".to_owned(),
            report: format!(
                "Block (Index: {}; Proof: {}): New Block mined: {}",
                block_index, proof, block_json
            ),
        })
    }
}

#[derive(Clone)]
pub struct MinerLink {
    addr: Addr<MiningWorker>,
}

impl MinerLink {
    pub fn new(addr: Addr<MiningWorker>) -> Self {
        Self { addr }
    }

    pub fn mine_block(
        &self,
    ) -> impl Future<Output = Result<MiningResponse, MiningError>> + 'static {
        let sender = self.addr.clone();
        async move {
            match sender.send(MiningMessage).await {
                Ok(rs) => rs,
                Err(e) => Err(MiningError {
                    status: String::from("failed"),
                    report: format!("Mining Error: '{:?}'", e),
                }),
            }
        }
    }
}

//==============================================================================
// Auxiliary Functions

pub async fn mine_block(link: &MinerLink) -> Result<MiningResponse, MiningError> {
    // Send Email Data message.
    // send() message returns Future object, that resolves to message result
    let mining_future = link.mine_block().await;

    match mining_future {
        Ok(rs) => {
            println!("Mining Result: '{:?}'", &rs);
            Ok(rs)
        }
        Err(e) => {
            println!("Mining Error: '{:?}'", &e);
            Err(e)
        }
    }
}
