use goud_engine::ecs::collision::{Contact, CollisionResponse, resolve_collision};
use goud_engine::core::math::Vec2;

fn main() {
    let contact = Contact::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0), // Normal points up
        0.1,
    );

    let vel_a = Vec2::new(10.0, -5.0);  // Moving right and down
    let vel_b = Vec2::zero();           // Static surface
    let inv_mass_a = 1.0;
    let inv_mass_b = 0.0;

    let response = CollisionResponse::character();
    let (delta_a, _) = resolve_collision(
        &contact,
        vel_a,
        vel_b,
        inv_mass_a,
        inv_mass_b,
        &response,
    );

    let new_vel_a = vel_a + delta_a;
    
    println!("Original vel_a: {:?}", vel_a);
    println!("Delta vel_a: {:?}", delta_a);
    println!("New vel_a: {:?}", new_vel_a);
    println!("vel_a.y = {}, new_vel_a.y = {}", vel_a.y, new_vel_a.y);
}
