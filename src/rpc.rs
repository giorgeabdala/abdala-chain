use crate::runtime::{Blockchain};
use rocket::{get, post, routes};
use rocket::serde::json::Json;
use crate::domain::block::Block;

/*#[get("/mine_block")]

fn mine_block(blockchain: &rocket::State<Blockchain>) -> Json<Block> {
    let previous_block = blockchain.get_previous_block();
    let previous_proof = previous_block.proof;
    let proof = blockchain.proof_of_work(previous_proof);
    let previous_hash = previous_block.hash();
    let block = blockchain.create_block(proof, previous_hash);
    Json(block)
}*/

#[get("/get_chain")]
fn get_chain(blockchain: &rocket::State<Blockchain>) -> Json<Vec<Block>> {
    let chain = blockchain.chain.lock().unwrap();
    Json(chain.clone())
}

#[get("/is_valid")]
fn is_valid(blockchain: &rocket::State<Blockchain>) -> Json<bool> {
    Json(blockchain.is_chain_valid())
}

pub async fn start_server() -> Result<(), rocket::Error> {
    let blockchain = Blockchain::new();
    rocket::build()
        .manage(blockchain)
        //.mount("/", routes![mine_block, get_chain, is_valid])
        .mount("/", routes![get_chain, is_valid])
        .launch()
        .await?;

    Ok(())



}