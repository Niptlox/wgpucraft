use super::universe::World;

pub trait System {
    /// Выполняет логику системы.
    fn run(&self, world: &mut World);
}
