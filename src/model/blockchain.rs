/*
* @author Bodo (Hugo) Barwich
* @version 2024-11-28
* @package Blockchain Exercise
* @subpackage Blockchain Structures

* This Module defines the Rust Structures to store the data of the Blockchain
*
*---------------------------------
* Requirements:
*/

use serde::{Deserialize, Serialize};
use serde_json::Error;
use sha256::digest;
//use num::pow;
use std::time::SystemTime;

//==============================================================================
// Structure Transaction Declaration

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
}

//==============================================================================
// Structure Block Declaration

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u32,
    pub proof: u64,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>,
}

//==============================================================================
// Structure Blockchain Declaration

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub transactions: Option<Vec<Transaction>>,
    pub nodes: Vec<String>,
}

//==============================================================================
// Structure Transaction Implementation

impl Transaction {
    /*----------------------------------------------------------------------------
     * Constructors
     */

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
    pub fn is_valid(&self) -> bool {
        !self.sender.is_empty() && !self.receiver.is_empty() && self.amount != 0f64
    }
}

//==============================================================================
// Structure Block Implementation

impl Default for Block {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        Self::new()
    }
}

impl Block {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> Self {
        Self::build_block(0, 0, "", None)
    }

    pub fn build_block(
        index: u64,
        proof: u64,
        previous_hash: &str,
        transactions: Option<Vec<Transaction>>,
    ) -> Self {
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs() as u32,
            Err(_) => 0 as u32,
        };
        let transactions = match transactions {
            Some(mut tx) => tx.drain(..).collect(),
            None => Vec::<Transaction>::new(),
        };

        Self {
            index: index,
            timestamp: timestamp,
            proof: proof,
            previous_hash: previous_hash.to_owned(),
            transactions: transactions,
        }
    }

    /*----------------------------------------------------------------------------
     * Administration Methods
     */

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(&self)
    }

    /*
    Cálculo del hash de un bloque.

    Arguments:
        - block: Identifica a un bloque de la Blockchain.
    Returns:
        - hash_block: Devuelve el hash del bloque
    */
    pub fn to_hash(&self) -> String {
        // Serialize it to a JSON string.
        let block_json = match self.to_json() {
            Ok(j) => j,
            Err(e) => {
                eprintln!(
                    "Block ({}): JSON formatting failed! Message: {:?}",
                    self.index, e
                );
                String::new()
            }
        };

        digest(block_json)
    }

    pub fn update_timestamp(&mut self, timestamp: Option<u32>) {
        self.timestamp = match timestamp {
            Some(t) => t,
            None => match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs() as u32,
                Err(_) => 0 as u32,
            },
        };
    }
}

//==============================================================================
// Structure Blockchain Implementation

impl Default for Blockchain {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> Self {
        let blockchain = Self {
            chain: Vec::<Block>::new(),
            transactions: None,
            nodes: Vec::<String>::new(),
        };
        // Generate Genesis Block
        //let _ = blockchain.proof_of_work();

        blockchain
    }

    /*----------------------------------------------------------------------------
     * Administration Methods
     */

    /*  Realización de una transacción.
        Arguments:
            - sender: Persona que hace la transacción
            - receiver: Persona que recibe la transacción
            - amount: Cantidad de criptomonedas enviadas

        Returns:
            - Índice del último bloque más uno
    */
    pub fn add_transaction_from_data(&mut self, sender: &str, receiver: &str, amount: f64) -> u64 {
        self.add_transaction(Transaction {
            sender: sender.to_owned(),
            receiver: receiver.to_owned(),
            amount: amount,
        })
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> u64 {
        match &mut self.transactions {
            Some(ref mut tx) => tx.push(transaction),
            None => {
                let mut tx = Vec::<Transaction>::new();

                tx.push(transaction);

                self.transactions = Some(tx);
            }
        }

        self.get_last_block_index() + 1
    }

    /*    Creación de un nuevo bloque.

          Arguments:
            - proof: Nounce del bloque actual.
            - previous_hash: Hash del bloque previo.

          Returns:
            - index: index del nuevo bloque creado.

    */
    pub fn build_block(&mut self, proof: u64, previous_hash: &str) -> u64 {
        let next_index = self.get_last_block_index() + 1;
        let block = Block::build_block(next_index, proof, previous_hash, self.transactions.take());

        self.chain.push(block);

        next_index
    }

    /*    Protocolo de concenso Proof of Work (PoW).
          Arguments:
            - previous_proof: Nounce del bloque previo.

          Returns:
            - new_proof: Devolución del nuevo nounce obtenido con PoW.
    */
    pub fn proof_of_work(&mut self) -> u64 {
        let last_block = self.get_last_block();
        let last_hash = match last_block {
            Some(b) => b.to_hash(),
            None => String::from("0"),
        };
        let next_index = match last_block {
            Some(b) => b.index + 1,
            None => 1,
        };
        let mut new_proof: u64 = 0;
        let mut new_block = Block::build_block(
            next_index,
            new_proof,
            last_hash.as_str(),
            self.transactions.take(),
        );
        let mut proof_matches = false;

        while !proof_matches {
            new_block.proof = new_proof;
            new_block.update_timestamp(None);

            if let Some(tx) = &mut self.transactions {
                tx.drain(..).for_each(|t| new_block.transactions.push(t));
            }

            let block_hash = new_block.to_hash();

            if block_hash.starts_with("0000") {
                proof_matches = true;
                println!("Hash (Proof: {}): '{}'", new_proof, block_hash);
            } else {
                new_proof += 1;
            }
        }

        // Store newly mined Block
        self.chain.push(new_block);

        new_proof
    }

    /*----------------------------------------------------------------------------
     * Consultation Methods
     */

    pub fn get_last_block_index(&self) -> u64 {
        if self.chain.len() > 0 {
            self.chain[self.chain.len() - 1].index
        } else {
            0
        }
    }

    pub fn get_last_block(&self) -> Option<&Block> {
        if self.chain.len() > 0 {
            Some(&self.chain[self.chain.len() - 1])
        } else {
            None
        }
    }
}

//==============================================================================
// Auxiliary Functions

/*    Protocolo de concenso Proof of Work (PoW).
      Arguments:
        - previous_proof: Nounce del bloque previo.

      Returns:
        - new_proof: Devolución del nuevo nounce obtenido con PoW.
*/
pub fn proof_of_work(previous_proof: u64) -> u64 {
    let mut new_proof: u64 = 1;
    let mut check_proof = false;

    while !check_proof {
        let hash_operation = digest(format!("{}", new_proof.pow(2) - previous_proof.pow(2)));

        if hash_operation.starts_with("0000") {
            check_proof = true;
            println!(
                "hash {} - {}: '{}'",
                new_proof, previous_proof, hash_operation
            );
        } else {
            new_proof += 1;
        }
    }

    new_proof
}
