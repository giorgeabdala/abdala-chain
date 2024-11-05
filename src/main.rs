mod runtime;
use runtime::Blockchain;

fn main() {
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
