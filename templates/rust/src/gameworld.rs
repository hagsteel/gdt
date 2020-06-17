use std::sync::Mutex;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, InputEvent, InputEventMouse, InputEventMouseButton, NativeClass, Node2D,
};
use lazy_static::lazy_static;
use legion::prelude::*;
use gdextras::input::InputEventExt;

use crate::input::{Keyboard, Keys, MouseButton, MousePos};

// -----------------------------------------------------------------------------
//     - World -
// -----------------------------------------------------------------------------

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(Universe::new().create_world());
}

pub fn with_world<F>(f: F)
where
    F: FnOnce(&mut World),
{
    let _ = WORLD.try_lock().map(|mut world| f(&mut world));
}

// -----------------------------------------------------------------------------
//     - System setup -
// -----------------------------------------------------------------------------
fn setup_physics_schedule() -> Schedule {
    let builder = Schedule::builder();
    // let builder = movement_systems(builder);
    builder.build()
}

fn setup_process_schedule() -> Schedule {
    let builder = Schedule::builder();
    builder.build()
}

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
pub struct Delta(pub f32);
pub struct PhysicsDelta(pub f32);

// -----------------------------------------------------------------------------
//     - Godot node -
// -----------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GameWorld {
    resources: Resources,
    process: Schedule,
    physics: Schedule,
}

#[methods]
impl GameWorld {
    pub fn _init(_owner: Node2D) -> Self {
        let mut resources = Resources::default();
        resources.insert(Delta(0.));
        resources.insert(PhysicsDelta(0.));
        resources.insert(MouseButton::Empty);
        resources.insert(MousePos::zero());

        Self {
            resources,
            process: setup_process_schedule(),
            physics: setup_physics_schedule(),
        }
    }

    #[export]
    pub fn _ready(&self, owner: Node2D) {}

    #[export]
    pub fn _unhandled_input(&self, owner: Node2D, event: InputEvent) {
        if event.action_pressed("ui_cancel") {
            unsafe { owner.get_tree().map(|mut tree| tree.quit(0)) };
        }

        // Mouse button
        if let Some(btn_event) = event.cast::<InputEventMouseButton>() {
            self.resources.get_mut::<MouseButton>().map(|mut btn| {
                *btn = MouseButton::from_event(btn_event);
            });
        }

        // Mouse pos
        if let Some(mouse_event) = event.cast::<InputEventMouse>() {
            self.resources.get_mut::<MousePos>().map(|mut pos| {
                pos.set_global(mouse_event.get_global_position());
            });
        }

        // Keyboard
        // if let Some(_) = event.cast::<InputEventKey>() {
        //     self.resources.get_mut::<Keyboard>().map(|mut key| {
        //         if event.is_action_pressed("Left".into(), false) {
        //             key.update(Keys::LEFT, true);
        //         } else if event.is_action_released("Left".into()) {
        //             key.update(Keys::LEFT, false);
        //         }

        //         if event.is_action_pressed("Right".into(), false) {
        //             key.update(Keys::RIGHT, true);
        //         } else if event.is_action_released("Right".into()) {
        //             key.update(Keys::RIGHT, false);
        //         }

        //         if event.is_action_pressed("Up".into(), false) {
        //             key.update(Keys::UP, true);
        //         } else if event.is_action_released("Up".into()) {
        //             key.update(Keys::UP, false);
        //         }

        //         if event.is_action_pressed("Down".into(), false) {
        //             key.update(Keys::DOWN, true);
        //         } else if event.is_action_released("Down".into()) {
        //             key.update(Keys::DOWN, false);
        //         }
        //     });
        // }
    }

    #[export]
    pub fn _process(&mut self, owner: Node2D, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);

        with_world(|world| {
            self.process.execute(world, &mut self.resources);
        });
    }

    #[export]
    pub fn _physics_process(&mut self, owner: Node2D, delta: f64) {
        self.resources
            .get_mut::<PhysicsDelta>()
            .map(|mut d| d.0 = delta as f32);

        with_world(|world| {
            self.physics.execute(world, &mut self.resources);
        });
    }
}
