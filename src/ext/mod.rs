pub mod extras {
    #[derive(Default, Eq, PartialEq, Clone, Debug)]
    pub struct MarkedResources(pub usize);

    #[derive(Default, Eq, PartialEq, Clone, Debug)]
    pub enum StateExample {
        #[default]
        A,
        B,
        C,
    }
}
