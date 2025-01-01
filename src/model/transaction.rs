/*
* @author Bodo (Hugo) Barwich
* @version 2024-12-30
* @package Blockchain Exercise
* @subpackage Transaction Structures

* This Module defines the Rust Structures to store the data of the Blockchain
*
*---------------------------------
* Requirements:
*/

use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

//==============================================================================
// Structure Transaction Declaration

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
}

//==============================================================================
// Structure Transaction Declaration

#[derive(Debug)]
pub struct MutexTransactionList {
    pub transaction_mutex: Mutex<Vec<Transaction>>,
}

/// Structure for Email Sending Errors
#[derive(Debug, PartialEq)]
pub struct TransactionMutexError {
    status: String,
    report: String,
}

//==============================================================================
// Structure Transaction Implementation

impl Transaction {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    /// Create a new Transaction.
    ///
    /// # Parameters:
    ///
    /// - `sender`: Person (Address) which started the transaction
    /// - `receiver`: Person (Address) which will receive the transaction
    /// - `amount`: Amount of Cryptocurrency send
    ///
    /// # Example:
    ///
    /// Create a `Transaction` for the Mining Reward
    /// ```
    ///    use blockchain_api::model::transaction::Transaction;
    ///
    ///    let reward = Transaction::from_data("Blockchain".to_owned(), "Miner".to_owned(), 10f64);
    /// ```
    pub fn from_data(sender: String, receiver: String, amount: f64) -> Self {
        Self {
            sender: sender,
            receiver: receiver,
            amount: amount,
        }
    }

    /*----------------------------------------------------------------------------
     * Consultation Methods
     */

    /// Check if a Transaction is valid.
    ///
    /// The fields `sender` and `receiver` must not be empty and the `amount` field must not be ` 0 `
    pub fn is_valid(&self) -> bool {
        !self.sender.is_empty() && !self.receiver.is_empty() && self.amount != 0f64
    }
}

//==============================================================================
// Structure MutexTransactionList Implementation

impl Default for MutexTransactionList {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        Self::new()
    }
}

impl MutexTransactionList {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> Self {
        Self {
            transaction_mutex: Mutex::new(Vec::<Transaction>::new()),
        }
    }

    pub fn from_vec(transactions: Vec<Transaction>) -> Self {
        Self {
            transaction_mutex: Mutex::new(transactions),
        }
    }

    /*----------------------------------------------------------------------------
     * Administration Methods
     */

    /// Register a Transaction by values.
    ///
    /// # Parameters:
    ///
    /// - `sender`: Person (Address) which started the transaction
    /// - `receiver`: Person (Address) which will receive the transaction
    /// - `amount`: Amount of Cryptocurrency send
    ///
    pub fn add_transaction_from_data(
        &self,
        sender: &str,
        receiver: &str,
        amount: f64,
    ) -> Result<(), TransactionMutexError> {
        self.add_transaction(Transaction {
            sender: sender.to_owned(),
            receiver: receiver.to_owned(),
            amount: amount,
        })
    }

    /// Register a Transaction by structure.
    ///
    /// # Parameters:
    ///
    /// - `transaction`: `Transaction` to be added. It will be published as soon as
    /// a new block is mined.
    ///
    pub fn add_transaction(&self, transaction: Transaction) -> Result<(), TransactionMutexError> {
        match self.transaction_mutex.lock() {
            Ok(mut guard) => {
                let transactions = guard.deref_mut();

                transactions.push(transaction);

                Ok(())
            }
            Err(e) => Err(TransactionMutexError {
                status: "failed".to_owned(),
                report: format!("Transaction List: Mutex Lock failed! Message: {:?}", e),
            }),
        }
    }

    /*----------------------------------------------------------------------------
     * Consultation Methods
     */

    pub fn into_vec(&self) -> Vec<Transaction> {
        match self.transaction_mutex.lock() {
            Ok(mut guard) => {
                let transactions = guard.deref_mut();
                let mut export = Vec::<Transaction>::with_capacity(transactions.len());

                transactions.drain(..).for_each(|t| export.push(t));

                export
            }
            Err(mut e) => {
                eprintln!("Transaction List: Mutex Lock failed! Message: {:?}", e);

                let transactions = e.get_mut();
                let mut export = Vec::<Transaction>::with_capacity(transactions.len());

                transactions.drain(..).for_each(|t| export.push(t));

                export
            }
        }
    }

    pub fn get_count(&self) -> usize {
        match self.transaction_mutex.lock() {
            Ok(guard) => {
                let transactions = guard.deref();

                transactions.len()
            }
            Err(e) => {
                eprintln!("Transaction List: Mutex Lock failed! Message: {:?}", e);

                let transactions = e.get_ref();

                transactions.len()
            }
        }
    }
}

//==============================================================================
// Unit Tests

/*
Recreating the Test Data:

    $ curl -v http://localhost:3100/add_transaction -d '{"sender":"sender1","receiver":"receiver1","amount":5.67}' | jq '.'

*/

#[test]
fn transaction_list_from_vec() {
    //-------------------------------------
    // Create a `MutexTransactionList` from a Vector

    let mut transactions = Vec::<Transaction>::with_capacity(3);

    transactions.push(Transaction {
        sender: "sender1".to_owned(),
        receiver: "receiver1".to_owned(),
        amount: 5.67f64,
    });
    transactions.push(Transaction {
        sender: "sender2".to_owned(),
        receiver: "receiver2".to_owned(),
        amount: 7.89107f64,
    });
    transactions.push(Transaction {
        sender: "sender3".to_owned(),
        receiver: "receiver3".to_owned(),
        amount: 9.101113f64,
    });

    let transaction_count = transactions.len();

    assert_eq!(transaction_count, 3);

    let transaction_mutex = MutexTransactionList::from_vec(transactions);

    let transaction_count = transaction_mutex.get_count();

    assert_eq!(transaction_count, 3);
}

#[test]
fn add_transaction_from_data() {
    //-------------------------------------
    // Create a `Transaction` from a data set
    // Transaction List does not need to be `mut` because it is a `Mutex`

    let transaction_mutex = MutexTransactionList::new();

    let transaction_count = transaction_mutex.get_count();

    assert_eq!(transaction_count, 0);

    let result = transaction_mutex.add_transaction_from_data("sender1", "receiver1", 5.67);

    assert_eq!(result.err(), None);

    let result = transaction_mutex.add_transaction_from_data("sender2", "receiver2", 7.89107);

    assert_eq!(result.err(), None);

    let result = transaction_mutex.add_transaction_from_data("sender3", "receiver3", 9.101113);

    assert_eq!(result.err(), None);

    let transaction_count = transaction_mutex.get_count();

    assert_eq!(transaction_count, 3);

    let transactions = transaction_mutex.into_vec();

    assert_eq!(transactions.len(), 3);

    let transaction_count = transaction_mutex.get_count();

    assert_eq!(transaction_count, 0);
}

#[test]
fn add_transaction_as_structure() {
    //-------------------------------------
    // Create a Transaction from a pre-built structure
    // Transaction List does not need to be `mut` because it is a `Mutex`

    let transaction_mutex = MutexTransactionList::new();

    let result = transaction_mutex.add_transaction(Transaction {
        sender: "sender1".to_owned(),
        receiver: "receiver1".to_owned(),
        amount: 5.67f64,
    });

    assert_eq!(result.err(), None);

    let result = transaction_mutex.add_transaction(Transaction {
        sender: "sender2".to_owned(),
        receiver: "receiver2".to_owned(),
        amount: 7.89107f64,
    });

    assert_eq!(result.err(), None);

    let result = transaction_mutex.add_transaction(Transaction {
        sender: "sender3".to_owned(),
        receiver: "receiver3".to_owned(),
        amount: 9.101113f64,
    });

    assert_eq!(result.err(), None);

    let transaction_count = transaction_mutex.get_count();

    assert_eq!(transaction_count, 3);
}
