pub mod arch_types
{
    use crate :: corrosive_engine :: auto_prelude :: prelude :: * ; use std ::
    collections :: HashSet; use std :: sync :: RwLock; #[derive(Copy, Clone)]
    pub struct macro_test0 < 'a >
    {
        ve0 : & 'a Vec < (LockedRef < Position3 >, Ref < Position2 >,) > ,
        rve0 : & 'a RwLock < HashSet < usize >> , ve1 : & 'a Vec <
        (LockedRef < Position3 >, Position3, Ref < Position2 >,) > , rve1 : &
        'a RwLock < HashSet < usize >> , pub len : usize, pub v_i : usize,
    } impl < 'a > macro_test0 < 'a >
    {
        pub fn
        new(ve0 : & 'a Vec < (LockedRef < Position3 >, Ref < Position2 >,) > ,
        rve0 : & 'a RwLock < HashSet < usize >> , ve1 : & 'a Vec <
        (LockedRef < Position3 >, Position3, Ref < Position2 >,) > , rve1 : &
        'a RwLock < HashSet < usize >> ,) -> Self
        {
            macro_test0
            { ve0, rve0, ve1, rve1, len : ve0.len() + ve1.len(), v_i : 0, }
        } pub fn remove(& self, mut index : usize)
        {
            if index < self.ve0.len()
            { self.rve0.write().unwrap().insert(index); return; }; index -=
            self.ve0.len(); if index < self.ve1.len()
            { self.rve1.write().unwrap().insert(index); return; }; index -=
            self.ve1.len();
        }
    } impl < 'a > Iterator for macro_test0 < 'a >
    {
        type Item = (& 'a LockedRef < Position3 > , & 'a Ref < Position2 > ,);
        fn next(& mut self) -> Option < Self :: Item >
        {
            let mut current_index = self.v_i.clone(); self.v_i += 1; if
            current_index < self.ve0.len()
            {
                return
                Some((& self.ve0 [current_index].0, & self.ve0
                [current_index].1,));
            }; current_index -= self.ve0.len(); if current_index <
            self.ve1.len()
            {
                return
                Some((& self.ve1 [current_index].0, & self.ve1
                [current_index].2,));
            }; current_index -= self.ve1.len(); None
        }
    } #[derive(Copy, Clone)] pub struct macro_test1 < 'a >
    {
        ve0 : & 'a Vec < (LockedRef < Position3 >, Ref < Position2 >,) > ,
        rve0 : & 'a RwLock < HashSet < usize >> , ve1 : & 'a Vec <
        (LockedRef < Position3 >, Position3, Ref < Position2 >,) > , rve1 : &
        'a RwLock < HashSet < usize >> , pub len : usize, pub v_i : usize,
    } impl < 'a > macro_test1 < 'a >
    {
        pub fn
        new(ve0 : & 'a Vec < (LockedRef < Position3 >, Ref < Position2 >,) > ,
        rve0 : & 'a RwLock < HashSet < usize >> , ve1 : & 'a Vec <
        (LockedRef < Position3 >, Position3, Ref < Position2 >,) > , rve1 : &
        'a RwLock < HashSet < usize >> ,) -> Self
        {
            macro_test1
            { ve0, rve0, ve1, rve1, len : ve0.len() + ve1.len(), v_i : 0, }
        } pub fn remove(& self, mut index : usize)
        {
            if index < self.ve0.len()
            { self.rve0.write().unwrap().insert(index); return; }; index -=
            self.ve0.len(); if index < self.ve1.len()
            { self.rve1.write().unwrap().insert(index); return; }; index -=
            self.ve1.len();
        }
    } impl < 'a > Iterator for macro_test1 < 'a >
    {
        type Item = (& 'a LockedRef < Position3 > , & 'a Ref < Position2 > ,);
        fn next(& mut self) -> Option < Self :: Item >
        {
            let mut current_index = self.v_i.clone(); self.v_i += 1; if
            current_index < self.ve0.len()
            {
                return
                Some((& self.ve0 [current_index].0, & self.ve0
                [current_index].1,));
            }; current_index -= self.ve0.len(); if current_index <
            self.ve1.len()
            {
                return
                Some((& self.ve1 [current_index].0, & self.ve1
                [current_index].2,));
            }; current_index -= self.ve1.len(); None
        }
    } #[derive(Copy, Clone)] pub struct macro_test2 < 'a >
    {
        ve0 : & 'a Vec < (LockedRef < Position3 >, Ref < Position2 >,) > ,
        rve0 : & 'a RwLock < HashSet < usize >> , ve1 : & 'a Vec <
        (LockedRef < Position3 >, Position3, Ref < Position2 >,) > , rve1 : &
        'a RwLock < HashSet < usize >> , ve2 : & 'a Vec <
        (LockedRef < Position3 >,) > , rve2 : & 'a RwLock < HashSet < usize >>
        , pub len : usize, pub v_i : usize,
    } impl < 'a > macro_test2 < 'a >
    {
        pub fn
        new(ve0 : & 'a Vec < (LockedRef < Position3 >, Ref < Position2 >,) > ,
        rve0 : & 'a RwLock < HashSet < usize >> , ve1 : & 'a Vec <
        (LockedRef < Position3 >, Position3, Ref < Position2 >,) > , rve1 : &
        'a RwLock < HashSet < usize >> , ve2 : & 'a Vec <
        (LockedRef < Position3 >,) > , rve2 : & 'a RwLock < HashSet < usize >>
        ,) -> Self
        {
            macro_test2
            {
                ve0, rve0, ve1, rve1, ve2, rve2, len : ve0.len() + ve1.len() +
                ve2.len(), v_i : 0,
            }
        } pub fn remove(& self, mut index : usize)
        {
            if index < self.ve0.len()
            { self.rve0.write().unwrap().insert(index); return; }; index -=
            self.ve0.len(); if index < self.ve1.len()
            { self.rve1.write().unwrap().insert(index); return; }; index -=
            self.ve1.len(); if index < self.ve2.len()
            { self.rve2.write().unwrap().insert(index); return; }; index -=
            self.ve2.len();
        }
    } impl < 'a > Iterator for macro_test2 < 'a >
    {
        type Item = (& 'a LockedRef < Position3 > ,); fn next(& mut self) ->
        Option < Self :: Item >
        {
            let mut current_index = self.v_i.clone(); self.v_i += 1; if
            current_index < self.ve0.len()
            { return Some((& self.ve0 [current_index].0,)); }; current_index
            -= self.ve0.len(); if current_index < self.ve1.len()
            { return Some((& self.ve1 [current_index].0,)); }; current_index
            -= self.ve1.len(); if current_index < self.ve2.len()
            { return Some((& self.ve2 [current_index].0,)); }; current_index
            -= self.ve2.len(); None
        }
    }
}