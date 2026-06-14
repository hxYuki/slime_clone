use bevy::prelude::*;

pub fn plugin<C: Component + Clone>(app: &mut App) {
    app.add_systems(Update, on_inserted::<C>);
}

fn on_inserted<C: Component + Clone>(
    mut commands: Commands,
    query: Query<(Entity, &C), Added<C>>,
    q_children: Query<&Children>,
) {
    query.iter().for_each(|(entity, component)| {
        q_children.iter_descendants(entity).for_each(|child| {
            commands.entity(child).insert(component.clone());
        });
    });
}
