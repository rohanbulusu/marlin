
#![allow(dead_code)]

mod colors;
mod entities;
mod marlin;

use winit::window::{WindowBuilder};
use winit::event_loop::EventLoop;

use colors::{BLUE, RED, WHITE};
use entities::{ShapeKind, Vertex};
use marlin::{MasterWindowState, SceneName};

#[tokio::main]
async fn main() {

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = MasterWindowState::new(window).await;

    state.add_shape(&SceneName::Home, &ShapeKind::Circle(500.0), vec![Vertex::new(0.0, 0.0, 0.0, BLUE)]);

    state.add_button(&SceneName::Home, &ShapeKind::Rectangle, vec![
        Vertex::new(-200.0, 50.0, 0.0, WHITE),
        Vertex::new(-200.0, -50.0, 0.0, WHITE),
        Vertex::new(200.0, -50.0, 0.0, WHITE),
        Vertex::new(200.0, 50.0, 0.0, WHITE)
    ], state.next_scene());

    state.add_shape(&SceneName::RootPicker, &ShapeKind::Circle(500.0), vec![Vertex::new(0.0, 0.0, 0.0, RED)]);

    state.add_button(&SceneName::RootPicker, &ShapeKind::Rectangle, vec![
        Vertex::new(-200.0, 50.0, 0.0, WHITE),
        Vertex::new(-200.0, -50.0, 0.0, WHITE),
        Vertex::new(200.0, -50.0, 0.0, WHITE),
        Vertex::new(200.0, 50.0, 0.0, WHITE)
    ], state.previous_scene());


    state.run(event_loop).await;
}