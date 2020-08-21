use bevy::prelude::*;
pub struct TestbedPlugin;

impl Plugin for TestbedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        super::net::build(app);
        super::ui::build(app);
        super::game::build(app);
    }
}
