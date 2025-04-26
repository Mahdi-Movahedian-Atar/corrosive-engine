/// Used as a return type for tasks.
/// T must be a tuple.
///
/// Example:
/// ```rust
/// #[task]
/// pub fn setup() -> (
///     RArch<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
///     RArch<(Locked<Position1>, LockedRef<Position3>)>,
/// ) {
///     let mut rng = rand::thread_rng();
///     let mut r1: RArch<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)> = RArch::default();
///     let mut r2: RArch<(Locked<Position1>, LockedRef<Position3>)> = RArch::default();
///
///     let random_number: u32 = rng.gen_range(0..10000);
///     for i in 0..10000 {
///         r1.add((
///             Locked::new(if (random_number == i) {
///                 Position1 { x: 10.0, y: 10.0 }
///             } else {
///                 Position1 { x: 2.0, y: 2.0 }
///             }),
///             Ref::new(Position2 { x: 2.0, y: 2.0 }),
///             LockedRef::new(Position3 { x: 2.0, y: 2.0 }),
///         ));
///     }
///     (r1, r2)
/// }
/// ```
pub struct RArch<T> {
    pub vec: Vec<T>,
}
impl<T> Default for RArch<T> {
    fn default() -> Self {
        RArch { vec: Vec::new() }
    }
}

impl<T> RArch<T> {
    /// Adds a single member to the archetype.
    pub fn add(&mut self, t: T) {
        self.vec.push(t);
    }
    /// Adds a multiple members to the archetype if the input value implements `IntoIterator` trait.
    pub fn add_multiple<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.vec.extend(items)
    }
    /// Gets a reference to the archetype members.
    pub fn get(&self) -> &Vec<T> {
        &self.vec
    }
}
