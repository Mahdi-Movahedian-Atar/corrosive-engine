# Task folder

1. All modules must be Directory modules. Modules files or nested modules within a file won't be detected by the engine.
   Modules must be public.
2. Use `task` to mark functions to be used by the engine.
3. Should tasks need to export something, they must be inside a tuple.
4. Use the `arch_types` macro to mark the arch types to be used by the engine.
5. Should tasks need to export something, they must be inside a tuple.
6. Tasks can only have `Res<T>`, `State<T>`, `Hierarchy<T>`, `Arch<(&T1,&T2,...).` and `DeltaTime` types as input and `RArch<(&T1,&T2,...)>`, Signal and Reset as output.

## Example:

```
#[task]
pub fn example(
    e1: Arch<(&LockedRef<Position3>,)>,
    e2: Arch<(&LockedRef<Position3>, &Ref<Position2>)>,
    e3: Arch<(&Ref<Position2>, &LockedRef<Position3>)>,
    e4: State<StateExample>,
    e5: Res<MarkedResources>,
) -> (
    RArch<(Ref<Position2>, LockedRef<Position3>)>,
    RArch<(Ref<Position2>, Position3, LockedRef<Position3>)>,
    RArch<(LockedRef<Position3>,)>,
    Signal,
    Reset,
) {
    let mut r1: RArch<(Ref<Position2>, LockedRef<Position3>)> = RArch::default();
    let mut r2: RArch<(Ref<Position2>, Position3, LockedRef<Position3>)> = RArch::default();
    let mut r3: RArch<((LockedRef<Position3>,))> = RArch::default();
    let mut signal = Signal::default();
    let mut reset = Reset::default();
    r1.add((
        Ref::new(Position2 { x: 1.0, y: 1.0 }),
        LockedRef::new(Position3 { x: 1.0, y: 1.0 }),
    ));
    r2.add((
        Ref::new(Position2 { x: 1.0, y: 1.0 }),
        Position3 { x: 1.0, y: 1.0 },
        LockedRef::new(Position3 { x: 1.0, y: 1.0 }),
    ));
    r3.add((LockedRef::new(Position3 { x: 1.0, y: 1.0 }),));
    signal.trigger("aa");
    reset.trigger();
    (r1, r2, r3, signal, reset)
}
```