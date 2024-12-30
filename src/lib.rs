/*
* @author Bodo (Hugo) Barwich
* @version 2024-11-30
* @package Blockchain Exercise
* @subpackage Blockchain API

* This Module defines the HTTP Interface for interacting with the Blockchain
*
*---------------------------------
* Requirements:
* - The Rust Crate "actix-web" must be installed
* - The Rust Crate "futures" must be installed
* - The Rust Crate "serde" must be installed
* - The Rust Crate "serde-json" must be installed
* - The Rust Crate "json" must be installed
*/

//#[macro_use]
extern crate json;

pub mod config;
pub mod miner;
pub mod model;

use actix::SyncArbiter;
use actix_web::middleware::Logger;
use actix_web::{error, web, App, Error, HttpResponse, HttpServer};
use futures_util::StreamExt;
use std::env;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use config::AppConfig;
use miner::{MinerLink, MiningWorker};
use model::blockchain::Blockchain;
use model::transaction::{MutexTransactionList, Transaction};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub title: String,
    pub statuscode: u16,
    pub page: String,
    pub description: String,
}

/// Handler to build the Home Page
pub async fn dispatch_home_page() -> HttpResponse {
    //------------------------
    // Project Description

    HttpResponse::Ok().json(ResponseData {
        title: String::from("Actix Blockchain API"),
        statuscode: 200,
        page: String::from("Home"),
        description: String::from(
            "Blockchain Exercise to simulate the workflow of a crypto-currency",
        ),
    })
}

/// Handler to add a Transaction to the Blockchain
pub async fn add_transaction(
    transaction_mutex: web::Data<MutexTransactionList>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    println!("got payload: '{:?}'", &body);

    // body is loaded, now we can deserialize serde-json
    match serde_json::from_slice::<Transaction>(&body) {
        Ok(request_transaction) => {
            println!("Transaction: {:?}", request_transaction);

            if !request_transaction.is_valid() {
                eprintln!("POST Transaction: Transaction is invalid");

                return Err(error::ErrorBadRequest("Transaction is invalid"));
            }

            match transaction_mutex.add_transaction(request_transaction) {
                Ok(_) => {
                    println!("Transactions: {:?}", transaction_mutex);

                    //------------------------
                    // Success Notfication

                    Ok(HttpResponse::Created().json(ResponseData {
                        title: String::from("Actix Blockchain API - Success"),
                        statuscode: 201,
                        page: String::from("Add Transaction"),
                        description: format!("Transactions: Transaction is queued for next block"),
                    }))
                }
                Err(e) => {
                    //------------------------
                    // Error Notfication

                    Ok(HttpResponse::InternalServerError().json(ResponseData {
                        title: String::from("Actix Blockchain API - Error"),
                        statuscode: 500,
                        page: String::from("Add Transaction"),
                        description: format!(
                            "Transactions: Transaction could not be added! Message: {:?}",
                            e
                        ),
                    }))
                }
            }
        }
        // Payload Parse failed
        Err(e) => {
            eprintln!("JSON parsing failed: {:?}", e);
            Err(error::ErrorBadRequest(e))
        }
    }
}

/// This Handler reads the Request and parses it into EmailData object with serde
pub async fn dispatch_mining_request(link: web::Data<MinerLink>) -> Result<HttpResponse, Error> {
    match miner::mine_block(&link).await {
        Ok(rs) => {
            println!("mining res: '{:?}'", rs);
            Ok(HttpResponse::Ok().json(rs)) // <- send response
        }
        Err(e) => {
            println!("mining error: '{:?}'", e);
            Err(error::ErrorBadRequest(format!(
                "Sending failed: '{:?}'\n",
                e
            )))
        }
    }
}

//==============================================================================
// Executing Section

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let config = AppConfig::from_file();

    println!("app config: {:?}", config);

    let component_name = match env::var("COMPONENT") {
        Ok(comp) => comp,
        Err(_) => "default".to_owned(),
    };

    let mut app_host = String::from("127.0.0.1");
    let app_port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3100".to_owned(),
    };

    app_host.push(':');
    app_host.push_str(app_port.as_str());

    println!(
        "Blockchain API '{}': launching at {} ...",
        component_name, app_host
    );

    let blockchain = web::Data::new(Mutex::new(Blockchain::new()));
    let transactions = web::Data::new(MutexTransactionList::new());

    //Clone the Blockchain and the Transaction Vector for the Mining Worker
    let worker_blockchain = blockchain.clone();
    let worker_transactions = transactions.clone();

    //Create 2 Mining Worker Instances
    let miner = SyncArbiter::start(config.miner_count as usize, move || {
        // Each Worker needs a copy of the reference to the Blockchain Data and
        // the Transaction Vector
        MiningWorker::with_data(worker_blockchain.clone(), worker_transactions.clone())
    });
    //Create 1 Mining Link Object
    let link = MinerLink::new(miner);

    HttpServer::new(move || {
        let app_config = web::Data::new(config.clone());
        let link_data = web::Data::new(link.clone());

        App::new()
            .app_data(blockchain.clone())
            .app_data(transactions.clone())
            .app_data(link_data)
            .app_data(web::JsonConfig::default().limit(MAX_SIZE)) // <- limit size of the payload (global configuration)
            .service(
                web::resource(app_config.web_root.as_str())
                    .route(web::get().to(dispatch_home_page)),
            )
            .service(
                web::resource(app_config.web_root.as_str().to_owned() + "add_transaction")
                    .route(web::post().to(add_transaction)),
            )
            .service(
                web::resource(app_config.web_root.as_str().to_owned() + "mine_block")
                    .route(web::get().to(dispatch_mining_request)),
            )
            /*            .service(
                            web::resource(app_config.web_root.as_str().to_owned() + "ping")
                                .route(web::get().to(dispatch_ping_request)),
                        )
            */
            .app_data(app_config)
            .wrap(Logger::default())
    })
    .bind(app_host)?
    .run()
    .await?;

    println!("Blockchain API '{}': finished.", component_name);

    Ok(())
}
