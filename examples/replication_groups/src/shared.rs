use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::utils::Duration;
use tracing::Level;

use lightyear::client::prediction::Predicted;
use lightyear::prelude::client::{Confirmed, Interpolated};
use lightyear::prelude::*;

use crate::protocol::Direction;
use crate::protocol::*;

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        // REFLECTION
        app.register_type::<PlayerId>();
        app.register_type::<PlayerPosition>();
        app.register_type::<PlayerParent>();
        app.register_type::<PlayerColor>();
        app.register_type::<TailPoints>();
        app.register_type::<TailLength>();
        if app.is_plugin_added::<RenderPlugin>() {
            app.add_systems(Startup, init);
            app.add_systems(Update, draw_snakes);
        }
    }
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// This system defines how we update the player's positions when we receive an input
pub(crate) fn shared_movement_behaviour(mut position: Mut<PlayerPosition>, input: &Inputs) {
    const MOVE_SPEED: f32 = 10.0;
    match input {
        Inputs::Direction(direction) => match direction {
            Direction::Up => position.y += MOVE_SPEED,
            Direction::Down => position.y -= MOVE_SPEED,
            Direction::Left => position.x -= MOVE_SPEED,
            Direction::Right => position.x += MOVE_SPEED,
        },
        _ => {}
    }
}

// This system defines how we update the player's tails when the head is updated
// Note: we only apply logic for the Predicted entity on the client (Interpolated is updated
// during interpolation, and Confirmed is just replicated from Server)
pub(crate) fn shared_tail_behaviour(
    player_position: Query<Ref<PlayerPosition>, Or<(With<Predicted>, With<ReplicationTarget>)>>,
    mut tails: Query<
        (&mut TailPoints, &PlayerParent, &TailLength),
        Or<(With<Predicted>, With<ReplicationTarget>)>,
    >,
) {
    for (mut points, parent, length) in tails.iter_mut() {
        let Ok(parent_position) = player_position.get(parent.0) else {
            error!("Tail entity has no parent entity!");
            continue;
        };
        // if the parent position didn't change, we don't need to update the tail
        // (also makes sure we don't trigger change detection for the tail! which would mean we add
        //  new elements to the tail's history buffer)
        if !parent_position.is_changed() {
            continue;
        }
        // Update the front if the head turned
        let (front_pos, front_dir) = points.0.front().unwrap().clone();
        // NOTE: we do not deal with diagonal directions in this example
        let front_direction = Direction::from_points(front_pos, parent_position.0);
        // if the head is going in a new direction, add a new point to the front
        if front_direction.map_or(true, |dir| dir != front_dir) {
            trace!(
                old_front_dir = ?front_dir,
                new_front_dir = ?front_direction,
                "creating new inflection point");
            let inflection_pos = match front_dir {
                Direction::Up | Direction::Down => Vec2::new(front_pos.x, parent_position.y),
                Direction::Left | Direction::Right => Vec2::new(parent_position.x, front_pos.y),
            };
            let new_front_dir = Direction::from_points(inflection_pos, parent_position.0).unwrap();
            points.0.push_front((inflection_pos, new_front_dir));
            trace!(?points, "new points");
        }

        // Update the back
        // remove the back points that are above the length
        points.shorten_back(parent_position.0, length.0);
    }
}

/// System that draws the boxes of the player positions.
/// The components should be replicated from the server to the client
pub(crate) fn draw_snakes(
    mut gizmos: Gizmos,
    players: Query<(&PlayerPosition, &PlayerColor), Without<Confirmed>>,
    tails: Query<(&PlayerParent, &TailPoints), Without<Confirmed>>,
) {
    for (parent, points) in tails.iter() {
        debug!("drawing snake with parent: {:?}", parent.0);
        let Ok((position, color)) = players.get(parent.0) else {
            error!("Tail entity has no parent entity!");
            continue;
        };
        // draw the head
        gizmos.rect(
            Vec3::new(position.x, position.y, 0.0),
            Quat::IDENTITY,
            Vec2::ONE * 20.0,
            color.0,
        );
        // draw the first line
        gizmos.line_2d(position.0, points.0.front().unwrap().0, color.0);
        if position.0.x != points.0.front().unwrap().0.x
            && position.0.y != points.0.front().unwrap().0.y
        {
            debug!("DIAGONAL");
        }
        // draw the rest of the lines
        for (start, end) in points.0.iter().zip(points.0.iter().skip(1)) {
            gizmos.line_2d(start.0, end.0, color.0);
            if start.0.x != end.0.x && start.0.y != end.0.y {
                debug!("DIAGONAL");
            }
        }
    }
}
