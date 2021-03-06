use amethyst::{
    core::{bundle::SystemBundle},
    core::{SystemDesc},
    ecs::{Read, System, SystemData, World, Write, DispatcherBuilder},
    shrev::{EventChannel, ReaderId}, 
    network::simulation::{DeliveryRequirement, UrgencyRequirement, NetworkSimulationEvent, TransportResource, NetworkSimulationTime},
    Result, 
};
use log::{info, error};

use crate::network::{Pack, Cmd};
use crate::resources::{ClientStatus, IO, AppConfig};

/// A simple system that sends a ton of messages to all connections.
/// In this case, only the server is connected.
#[derive(Debug)]
pub struct ClientSystemBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for ClientSystemBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            ClientSystemDesc::default().build(world),
            "client_system",
            &[],); Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ClientSystemDesc;

/// A simple system that receives a ton of network events.
impl<'a, 'b> SystemDesc<'a, 'b, ClientSystem> for ClientSystemDesc {
    fn build(self, world: &mut World) -> ClientSystem {
        // Creates the EventChannel<NetworkEvent> managed by the ECS.
        <ClientSystem as System<'_>>::SystemData::setup(world);
        // Fetch the change we just created and call `register_reader` to get a
        // ReaderId<NetworkEvent>. This reader id is used to fetch new events from the network event
        // channel.
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        ClientSystem::new(reader)
    }
}

pub struct ClientSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl ClientSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for ClientSystem {
    type SystemData = (
        Write<'a, ClientStatus>, 
        Read<'a, NetworkSimulationTime>,
        Write<'a, TransportResource>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
        Write <'a, IO>,
        Read<'a, AppConfig>,
    );

    fn run(&mut self, (mut status, sim_time, mut net, channel, mut io, conf): Self::SystemData) {
        if sim_time.should_send_message_now() {
            if !status.connected {
                info!("We are not connected, ready player 1");
                let proof = format!("{} 1580235330 SignatureHere", conf.player_name);
                let p = Pack::new(Cmd::Connect(proof.to_string()), 0, None);  
                net.send_with_requirements(conf.server_ip.parse().unwrap(), &p.to_bin(), DeliveryRequirement::ReliableSequenced(None), UrgencyRequirement::OnTick);
                status.connected = true;
            }
            else {
                for resp in io.o.pop() {
                    net.send_with_requirements(conf.server_ip.parse().unwrap(), &resp.to_bin(), DeliveryRequirement::ReliableSequenced(None), UrgencyRequirement::OnTick);
                }
            }
        }

        // Incoming packets
        for event in channel.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(_addr, payload) => {
                    if *payload != b"ok".to_vec() {
                        let pl =  Pack::from_bin(payload.to_vec());
                        info!("Payload: {:?}", pl);
                        io.i.push(pl); // Add the pack to the IO vector
                    }
                }
                NetworkSimulationEvent::Connect(addr) => info!("New client connection: {}", addr),
                NetworkSimulationEvent::Disconnect(addr) => {
                    info!("Server Disconnected: {}", addr);
                }
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                _ => {}
            }
        }
    }
}
