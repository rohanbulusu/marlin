mod marlin;
mod entity;

use marlin::{Marlin};
use crate::entity::{Entity, Point};

#[tokio::main]
async fn main() {

    let mut window = Marlin::new().await;

    let tri = Entity::from_points(vec![
        Point::new(0.0, 0.5, [1.0, 0.0, 0.0]),
        Point::new(-0.5, -0.5, [0.0, 1.0, 0.0]),
        Point::new(0.5, -0.5, [0.0, 0.0, 1.0]),
    ]);

    let weird = Entity::from_points(vec![
        Point::new(1.0, 0.5, [1.0, 0.0, 0.0]),
        Point::new(-0.5, -0.5, [0.0, 1.0, 0.0]),
        Point::new(0.5, -0.5, [0.0, 0.0, 1.0]),
    ]);

    window.add_entity(tri);
    window.add_entity(weird);
    window.draw_point(0.5, 0.5, [1.0, 0.5, 0.2]);

    window.run().await;

}
