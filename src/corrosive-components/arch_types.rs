pub mod arch_types
{
    use crate :: corrosive_engine :: auto_prelude :: prelude :: * ; use std ::
    collections :: HashSet; use std :: sync :: RwLock; #[derive(Copy, Clone)]
    pub struct macro_test0 < 'a > { pub len : usize, pub v_i : usize, } impl <
    'a > macro_test0 < 'a >
    {
        pub fn new() -> Self { macro_test0 { len : v_i : 0, } } pub fn
        remove(& self, mut index : usize) {}
    } impl < 'a > Iterator for macro_test0 < 'a >
    {
        type Item = (& 'a Ref < Position2 > , & 'a LockedRef < Position3 > ,);
        fn next(& mut self) -> Option < Self :: Item >
        { let mut current_index = self.v_i.clone(); self.v_i += 1; None }
    } #[derive(Copy, Clone)] pub struct macro_test1 < 'a >
    { pub len : usize, pub v_i : usize, } impl < 'a > macro_test1 < 'a >
    {
        pub fn new() -> Self { macro_test1 { len : v_i : 0, } } pub fn
        remove(& self, mut index : usize) {}
    } impl < 'a > Iterator for macro_test1 < 'a >
    {
        type Item = (& 'a Ref < Position2 > , & 'a LockedRef < Position3 > ,);
        fn next(& mut self) -> Option < Self :: Item >
        { let mut current_index = self.v_i.clone(); self.v_i += 1; None }
    } #[derive(Copy, Clone)] pub struct macro_test2 < 'a >
    { pub len : usize, pub v_i : usize, } impl < 'a > macro_test2 < 'a >
    {
        pub fn new() -> Self { macro_test2 { len : v_i : 0, } } pub fn
        remove(& self, mut index : usize) {}
    } impl < 'a > Iterator for macro_test2 < 'a >
    {
        type Item = (& 'a LockedRef < Position3 > ,); fn next(& mut self) ->
        Option < Self :: Item >
        { let mut current_index = self.v_i.clone(); self.v_i += 1; None }
    }
}