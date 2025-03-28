pub fn
macro_test(b : corrosive_ecs_core :: ecs_core :: Arch <
(& LockedRef < Position3 > ,) > , a : corrosive_ecs_core :: ecs_core :: Arch <
(& LockedRef < Position3 > , & Ref < Position2 > ,) > , aa :
corrosive_ecs_core :: ecs_core :: Arch <
(& Ref < Position2 > , & LockedRef < Position3 > ,) > , c : corrosive_ecs_core
:: ecs_core :: Res < MarkedResources > , d : corrosive_ecs_core :: ecs_core ::
State < StateExample > ,) ->
(Vec < (LockedRef < Position3 > , Ref < Position2 > ,) > , Vec <
(LockedRef < Position3 > , Position3, Ref < Position2 > ,) > , Vec <
(LockedRef < Position3 > ,) > , bool, bool,)
{
    let mut engine_signal_trigger : bool = false; let mut
    engine_trigger_signal0 : bool = false; let mut engine_add_arch2 : Vec <
    (LockedRef < Position3 > ,) > = Vec :: new(); let mut engine_add_arch1 :
    Vec < (LockedRef < Position3 > , Position3, Ref < Position2 > ,) > = Vec
    :: new(); let mut engine_add_arch0 : Vec <
    (LockedRef < Position3 > , Ref < Position2 > ,) > = Vec :: new();
    engine_add_arch0.push((LockedRef :: new(Position3 { x : 1.0, y : 1.0 }),
    Ref :: new(Position2 { x : 1.0, y : 1.0 }),));
    engine_add_arch1.push((LockedRef :: new(Position3 { x : 1.0, y : 1.0 }),
    Position3 { x : 1.0, y : 1.0 }, Ref ::
    new(Position2 { x : 1.0, y : 1.0 }),));
    engine_add_arch2.push((LockedRef ::
    new(Position3 { x : 1.0, y : 1.0 }),)); engine_trigger_signal0 = true;
    engine_signal_trigger = true; return
    (engine_add_arch0, engine_add_arch1, engine_add_arch2,
    engine_trigger_signal0, engine_signal_trigger,);
}