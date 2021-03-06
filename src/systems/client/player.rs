use amethyst::core::{Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Read, Write, Entities, Entity, System, SystemData, WriteStorage};
use amethyst::input::InputHandler;
use amethyst::renderer::SpriteRender;

use std::time::Instant;
use log::info;

use crate::{
    components::{PlayerComponent, Action},
    key_bindings::{MovementBindingTypes, AxisBinding},
    map::{Room},
    network::{Pack, Cmd},
    resources::{IO, SpritesContainer},
    constants,
};


#[derive(SystemDesc)]
pub struct PlayerSystem{ 
    pub p1: Option<Entity>,
    pub timer: Option<Instant>,
    pub p1_name: String,
}

impl<'s> System<'s> for PlayerSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, PlayerComponent>,
        WriteStorage<'s, SpriteRender>,
        Write<'s, IO>,
        Write<'s, Room>,
        Entities<'s>,
        Read<'s, InputHandler<MovementBindingTypes>>,
        Read<'s, SpritesContainer>,
    );
 
    fn run(&mut self, (mut transforms, mut players, mut sprite_renders, mut io, room, entities, input, s): Self::SystemData) {
        for element in io.i.pop() {
            match &element.cmd {
                Cmd::InsertPlayer(play) =>  {
                    // If it's yourself
                    if play.name == self.p1_name { 
                        info!("Inserting Player 1"); 
                        let player = PlayerComponent::new(play.clone());
                        self.p1 = Some(entities
                            .build_entity()
                            .with(player.trans(), &mut transforms)
                            .with(player.get_orientated(&s.sprites), &mut sprite_renders)
                            .with(player, &mut players) 
                            .build());
                        self.timer = Some(Instant::now());
                    }
                    else {
                        info!("Inserting Player"); 
                        let player = PlayerComponent::new(play.clone());
                        entities
                            .build_entity()
                            .with(player.trans(), &mut transforms)
                            .with(player.get_orientated(&s.sprites), &mut sprite_renders)
                            .with(player, &mut players) 
                            .build();
                    }
                },
                _ => io.i.push(element), 
            }
        }
        
        match self.p1 {
            Some(p1) => {
                let now = Instant::now();
                let player = players.get_mut(p1).unwrap();

                if now.duration_since(self.timer.unwrap()).as_millis() >= constants::MOVEMENT_DELAY_MS {
                    let horizontal = input
                        .axis_value(&AxisBinding::Horizontal)
                        .unwrap_or(0.0);
                    let vertical = input
                        .axis_value(&AxisBinding::Vertical)
                        .unwrap_or(0.0);
                    
                    if horizontal == 0. && vertical == 0. {
                        return;
                    }
                    
                    let tr = transforms.get_mut(p1).unwrap(); 

                    player.update_orientation(&horizontal, &vertical);
                    self.timer = Some(now.clone());
                    sprite_renders.insert(p1, player.get_orientated(&s.sprites)).expect("Failed to insert orientated player!");

                    if room.allowed_move(tr, &player.orientation) {
                        player.walk(); // Walk one step in forward direction
                        tr.set_translation_xyz(player.x(), player.y(), player.z()); 
                        io.o.push(Pack::new(Cmd::Action(Action::Move(player.orientation.clone())), 0, None));
                    }
                }
            },
            None => ()
        }
    }
}
