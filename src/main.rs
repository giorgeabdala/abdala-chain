mod runtime;
mod rpc;

use runtime::Blockchain;

#[tokio::main]
async fn main() {
    run_rpc().await;


}

fn run_chain()  {
    println!("ABDALA CHAIN - A simple blockchain implementation in Rust");

    let chain = Blockchain::new();
    let mut previous_block = chain.get_previous_block().clone();

    let mut proof = chain.proof_of_work(previous_block.proof);
    let mut block = chain.create_block(proof, previous_block.previous_hash.clone());

    while true {
        println!("Block {} has been added to the chain", block.index);
        println!("Hash: {}", chain.hash(&block));
        println!("Proof: {}", block.proof);
        println!("Previous Hash: {}", block.previous_hash);
        println!("Timestamp: {}", block.timestamp);
        println!("\n");
        proof = chain.proof_of_work(block.proof);
        block = chain.create_block(proof, chain.hash(&block));

    }

}

async fn run_rpc() {
    rpc::start_server().await.unwrap();
    println!("RPC server started at http://localhost:8000");

}
