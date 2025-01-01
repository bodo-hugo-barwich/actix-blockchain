#[cfg(test)]
mod tests {
    use actix::sync::SyncArbiter;
    use actix_web::{http::header::ContentType, test, web, App};

    use std::sync::Mutex;

    use blockchain_api::miner::{MinerLink, MiningResponse, MiningWorker};
    use blockchain_api::model::blockchain::Blockchain;
    use blockchain_api::model::transaction::{MutexTransactionList, Transaction};
    use blockchain_api::{
        add_transaction, dispatch_home_page, dispatch_mining_request, ResponseData,
    };

    #[actix_rt::test]
    async fn test_home() {
        let mut app =
            test::init_service(App::new().route("/", web::get().to(dispatch_home_page))).await;
        let req = test::TestRequest::with_header("content-type", ContentType::json()).to_request();

        let resp = test::call_service(&mut app, req).await;

        println!("home hdrs: '{:?}'", resp);

        assert!(resp.status().is_success());

        let response: ResponseData = test::read_body_json(resp).await;

        println!("send bdy: '{:?}'", response);

        assert_eq!(response.page.as_str(), "Home");
        assert_eq!(response.statuscode, 200);
    }

    #[actix_rt::test]
    async fn test_add_transaction() {
        let transactions = web::Data::new(MutexTransactionList::new());

        let mut app = test::init_service(
            App::new()
                .app_data(transactions.clone())
                .route("/add_transaction", web::post().to(add_transaction)),
        )
        .await;

        let transaction = Transaction {
            sender: String::from("sender1"),
            receiver: String::from("receiver1"),
            amount: 11.1317f64,
        };

        let req = test::TestRequest::post()
            .uri("/add_transaction")
            .set_json(&transaction)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        println!("add tx hdrs: '{:?}'", resp);

        assert!(resp.status().is_success());

        let response: ResponseData = test::read_body_json(resp).await;

        println!("send bdy: '{:?}'", response);

        assert_eq!(response.page.as_str(), "Add Transaction");
        assert_eq!(response.statuscode, 201);
    }

    #[actix_rt::test]
    async fn test_mining() {
        let blockchain = web::Data::new(Mutex::new(Blockchain::new()));
        let transactions = web::Data::new(MutexTransactionList::new());

        //Clone the Blockchain and the Transaction Vector for the Mining Worker
        let worker_blockchain = blockchain.clone();
        let worker_transactions = transactions.clone();

        //Create 1 Mining Worker Instances
        let miner = SyncArbiter::start(1, move || {
            // Each Worker needs a copy of the reference to the Blockchain Data and
            // the Transaction Vector
            MiningWorker::with_data(worker_blockchain.clone(), worker_transactions.clone())
        });
        //Create 1 Mining Link Object
        let link = MinerLink::new(miner);

        let mut app = test::init_service(
            App::new()
                .app_data(blockchain.clone())
                .app_data(transactions.clone())
                .app_data(web::Data::new(link.clone()))
                .route("/mine_block", web::get().to(dispatch_mining_request)),
        )
        .await;

        let req = test::TestRequest::get().uri("/mine_block").to_request();
        let resp = test::call_service(&mut app, req).await;

        println!("mining hdrs: '{:?}'", resp);

        assert!(resp.status().is_success());

        let response: MiningResponse = test::read_body_json(resp).await;

        println!("send bdy: '{:?}'", response);

        assert_eq!(response.status.as_str(), "success");
    }
}
