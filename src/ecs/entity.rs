/// Уникальный идентификатор сущности в мире.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(usize);

impl Entity {
    /// Создаёт сущность с заданным ID.
    /// (На практике только `World` должен создавать сущности).
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Возвращает числовой ID сущности.
    pub fn id(&self) -> usize {
        self.0
    }
}
