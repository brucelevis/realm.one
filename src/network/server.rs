use crate::network::{Pack, Cmd};
use log::info;

use std::{
    fs::File,
};

use std::io::Read;
use crate::components::PlayerInfo;

/// Send the map to the client
fn welcome(proof: String) -> Option<Pack> {
    info!("Player Connected proof: {}, sending map!", proof);
    let fname = "resources/maps/townCompress2.tmx";
    let mut file = File::open(&fname.to_string()).expect("Unable to open map file"); 
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to convert to string");
    Some(Pack::new(Cmd::TransferMap(fname.to_string(), contents.to_string()), 0))
}

fn send_player() -> Option<Pack> {
    info!("Client has recived the map, inserting players!");
    Some(Pack::new(Cmd::CreatePlayer(Vec::<PlayerInfo>::new()), 0))
}

pub fn handle(bin: Vec<u8>) -> Option<Pack> {
    let pk = Pack::from_bin(bin);
    info!("{:?}", pk);

    match pk.cmd {
        Cmd::Nothing              => None,
        Cmd::TransferMap(..)      => None, 
        Cmd::RecivedMap           => send_player(),
        Cmd::Connect(proof)       => welcome(proof),
        Cmd::CreatePlayer(..)     => None,
    }
}