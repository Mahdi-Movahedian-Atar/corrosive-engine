pub mod other_tasks;

use crate::comp::sub::{MarkedResources, Position3, Position4, StateExample};
use crate::comp::{test, Position1, Position2};
use crate::corrosive_engine;
use corrosive_2d::comp::camera2d::{ActiveCamera2D, Camera2D};
use corrosive_2d::comp::sprite2d::Sprite2D;
use corrosive_2d::comp::{sprite2d, Position2D, RendererMeta2D};
use corrosive_2d::material2d::StandardMaterial2D;
use corrosive_2d::position2d_operations::Move2D;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{
    Arch, DeltaTime, Hierarchy, Locked, LockedRef, Member, RArch, Ref, Reference, Res, Reset,
    Signal, State,
};
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::color::Color;
use corrosive_ecs_renderer_backend::comp::{StarchMode, WindowOptions};
use corrosive_ecs_renderer_backend::winit::keyboard::KeyCode;
use corrosive_events::comp::Inputs;
use pixil::color_palette::ColorRange;
use pixil::comp::camera::{ActivePixilCamera, PixilCamera};
use pixil::comp::dynamic::PixilDynamicObject;
use pixil::comp::position_pixil::PositionPixil;
use pixil::glam::{Quat, Vec3};
use pixil::material::{PixilDefaultMaterial, PixilMaterial};
use pixil::position_operations::MovePixil;
use pixil::task::renderer::COLOR_PALLET;
use rand::Rng;
use std::iter::Map;
use std::process::id;
use std::vec::IntoIter;

#[task]
pub fn pixil_test(
    h: Hierarchy<PositionPixil>,
    ac: Res<ActivePixilCamera>,
    window_option: Res<WindowOptions>,
) -> (
    RArch<(PixilDynamicObject, Member<PositionPixil>)>,
    RArch<(LockedRef<PixilCamera>, Member<PositionPixil>)>,
) {
    let mut r: RArch<(PixilDynamicObject, Member<PositionPixil>)> = RArch::default();
    let mut r2: RArch<(LockedRef<PixilCamera>, Member<PositionPixil>)> = RArch::default();

    window_option.f_write().starch_mode = StarchMode::AspectRatio(16.0 / 8.0);

    COLOR_PALLET.set_palette(
        0,
        vec![
            ColorRange {
                size: 64,
                color: Color::from_hex("e8b494"),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 64,
                color: Color::from_hex("f83800"),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 64,
                color: Color::from_hex("7d2f01"),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 64,
                color: Color::from_hex("000000"),
                transition_type: Default::default(),
            },
            /*ColorRange {
                size: 128,
                color: Color::RGB(0.6, 0.0, 0.0),
                transition_type: Default::default(),
            },*/
            /*ColorRange {
                size: 128,
                color: Color::RGB(0.0, 0.0, 0.0),
                transition_type: Default::default(),
            },*/
        ],
    );
    /*COLOR_PALLET.set_palette(
        1,
        vec![
            ColorRange {
                size: 64,
                color: Color::RGB(0.0, 1.0, 0.0),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 128,
                color: Color::RGB(0.0, 0.5, 0.0),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 64,
                color: Color::RGB(0.0, 0.0, 0.0),
                transition_type: Default::default(),
            },
        ],
    );
    COLOR_PALLET.set_palette(
        2,
        vec![
            ColorRange {
                size: 64,
                color: Color::RGB(0.0, 0.0, 1.0),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 128,
                color: Color::RGB(0.0, 0.0, 0.5),
                transition_type: Default::default(),
            },
            ColorRange {
                size: 64,
                color: Color::RGB(0.0, 0.0, 0.1),
                transition_type: Default::default(),
            },
        ],
    );*/

    let a = h.new_entry(PositionPixil::new(
        Vec3::new(0.0, -1.0, -1.0),
        Quat::IDENTITY,
        Vec3::new(0.1, 0.1, 0.1),
    ));
    let b = h.new_entry(PositionPixil::new(
        Vec3::new(0.0, 0.0, 0.0),
        Quat::IDENTITY,
        Vec3::new(1.0, 1.0, 1.0),
    ));
    let c = LockedRef::new(PixilCamera {
        fov: 120.0_f32.to_radians(),
        near: 0.01,
        far: 100.0,
    });
    ac.f_write().new(&b, &c);

    r.add((
        PixilDynamicObject::new(
            AssetServer::load("assets/test.obj"),
            &AssetServer::add(1, || Ok(PixilDefaultMaterial::new())),
            &a,
            "test",
        ),
        a,
    ));
    r2.add((c, b));

    (r, r2)
}
#[task]
pub fn rotate_model(r: Arch<(&PixilDynamicObject, &Member<PositionPixil>)>, delta_time: DeltaTime) {
    for i in r.iter() {
        MovePixil::start(i.1)
            .rotate_around_global((1.0 * delta_time) as f32, Vec3::Y)
            .finish();
    }
}
#[task]
pub fn test2_0(
    position: Hierarchy<Position2D>,
    active_camera2d: Res<ActiveCamera2D>,
    active_camera: Res<ActiveCamera2D>,
) -> (
    RArch<(Member<Position2D>, RendererMeta2D, Sprite2D)>,
    RArch<(Member<Position2D>, LockedRef<Camera2D>)>,
) {
    let mut a: RArch<(Member<Position2D>, RendererMeta2D, Sprite2D)> = RArch::default();
    let mut b: RArch<(Member<Position2D>, LockedRef<Camera2D>)> = RArch::default();
    let new_position = position.new_entry(Position2D::new());
    Move2D::start(&new_position)
        .set_scale_local(8.0, 8.0)
        .set_transition_local(0.0, 0.0)
        .finish();

    let camera_position = position.new_entry(Position2D::default());
    let camera = LockedRef::new(Camera2D {
        right_boundary: Some(2.0),
        top_boundary: Some(2.0),
        left_boundary: Some(-2.0),
        bottom_boundary: Some(-2.0), /*
                                     right_boundary: Some(1.0),
                                     top_boundary: Some(2.0),
                                     bottom_boundary: Some(-2.0),*/
        /*min_zoom: Some(0.1),
        max_zoom: Some(1.9),*/
        ..Default::default()
    });
    Move2D::start(&camera_position)
        .transition_local(1.0, 0.0)
        .set_scale_local(1.0, 1.0)
        //.rotate_local(0.5)
        .finish();
    active_camera
        .f_write()
        .set_camera(&camera, &camera_position);
    b.add((camera_position, camera));

    let rect2d = Sprite2D::new(AssetServer::load("assets/default.jpg"), [0.0, 0.0]);
    let meta = RendererMeta2D::new(
        &AssetServer::add(static_hasher!("default"), || {
            Ok(StandardMaterial2D::new(Default::default()))
        }),
        &rect2d,
        &new_position,
        &active_camera2d,
    );
    a.add((new_position, meta, rect2d));
    (a, b)
}

#[task]
pub fn move_camera(active_camera: Res<ActiveCamera2D>, input: Res<Inputs>, delta_time: DeltaTime) {
    let cam_lock = active_camera.f_read();
    let cam_pos = match cam_lock.get_camera_position() {
        Some(p) => p,
        None => return,
    };
    let input = input.f_read();
    if input.is_key_held(KeyCode::KeyW) {
        Move2D::start(&cam_pos)
            .transition_local(0.0, (1.0 * delta_time) as f32)
            .finish();
    }
    if input.is_key_held(KeyCode::KeyS) {
        Move2D::start(&cam_pos)
            .transition_local(0.0, (-1.0 * delta_time) as f32)
            .finish();
    }
    if input.is_key_held(KeyCode::KeyD) {
        Move2D::start(&cam_pos)
            .transition_local((1.0 * delta_time) as f32, 0.0)
            .finish();
    }
    if input.is_key_held(KeyCode::KeyA) {
        Move2D::start(&cam_pos)
            .transition_local((-1.0 * delta_time) as f32, 0.0)
            .finish();
    }
    Move2D::start(&cam_pos)
        .scale_global(input.get_mouse_wheel() * 0.1, input.get_mouse_wheel() * 0.1)
        .finish();
}

#[task]
pub fn setup() -> (
    RArch<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
    RArch<(Locked<Position1>, LockedRef<Position3>)>,
) {
    let mut rng = rand::thread_rng();
    let mut r1: RArch<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)> = RArch::default();
    let mut r2: RArch<(Locked<Position1>, LockedRef<Position3>)> = RArch::default();

    let random_number: u32 = rng.gen_range(0..10000);
    for i in 0..10000 {
        r1.add((
            Locked::new(if (random_number == i) {
                Position1 { x: 10.0, y: 10.0 }
            } else {
                Position1 { x: 2.0, y: 2.0 }
            }),
            Ref::new(Position2 { x: 2.0, y: 2.0 }),
            LockedRef::new(Position3 { x: 2.0, y: 2.0 }),
        ));
    }
    (r1, r2)
}

#[task]
pub fn macro_test(
    b: Arch<(&LockedRef<Position3>,)>,
    a: Arch<(&LockedRef<Position3>, &Ref<Position2>)>,
    aa: Arch<(&Ref<Position2>, &LockedRef<Position3>)>,
    d: State<StateExample>,
    c: Res<MarkedResources>,
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
pub fn wrapper_macro_test<'a>(
    b: Arch<(&LockedRef<Position3>,)>,
    a: Arch<(&LockedRef<Position3>, &Ref<Position2>)>,
    aa: Arch<(&Ref<Position2>, &LockedRef<Position3>)>,
    d: State<StateExample>,
    c: Res<MarkedResources>,
) -> (
    Map<
        IntoIter<(Ref<Position2>, LockedRef<Position3>)>,
        fn((Ref<Position2>, LockedRef<Position3>)) -> (LockedRef<Position3>, Ref<Position2>),
    >,
) {
    let o = macro_test(b, a, aa, d, c);
    (o.0.vec.into_iter().map(|(a, b)| (b, a)),)
}

#[task]
pub fn setup1() {
    for _i in 0..10000 {}
}
#[task]
pub fn setup2() {
    for _i in 0..10000 {}
}
#[task]
pub fn update_task(inp: Arch<(&dyn test,)>, res: Res<MarkedResources>, delta_time: DeltaTime) {
    let mut mark: usize = 0;
    for x in inp.iter() {
        if x.0.get_num() == 10.0 {
            res.write().unwrap().0 = mark.clone();
            break;
        }
        mark += 1;
    }
    println!("{:?},{}", inp.len(), delta_time);
    for i in 500..inp.len() {
        inp.remove(i);
    }
}
#[task]
pub fn update_task_signal(sat: State<StateExample>) {
    let mut rng = rand::thread_rng();
    let random_number: usize = rng.gen_range(0..10000);

    let mut mark: usize = 0;
    /*for x in &inp {
        if mark == random_number {
            *x.0.write().unwrap() = Position1 { x: 10.0, y: 10.0 };
            break;
        }
        println!("{:?}", sat.read().unwrap());
        mark += 1;
    }*/
}

#[task]
pub fn fixed_task() {
    println!("Fixed")
}
