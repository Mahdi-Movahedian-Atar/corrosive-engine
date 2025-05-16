use crate::comp::position_pixil::PositionPixil;
use corrosive_ecs_core::ecs_core::{Member, Reference};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use std::ops::Add;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub struct MovePixil<'a> {
    member: &'a Member<PositionPixil>,
    step: Step,
    lock: Lock<'a>,
}
pub struct ExpiredValue {}
enum Lock<'a>{
    Lock(RwLockWriteGuard<'a, Reference<PositionPixil>>),
    NotLock
}
impl Lock<'_> {
    fn drop(mut self) {
        self = Lock::NotLock
    }
    fn unwrap(&mut self) -> & mut PositionPixil  {
        match self {
            Lock::Lock(v) => {v.unwrap_mut()}
            Lock::NotLock => {panic!("Lock does not exists")}
        }
    }
}
#[derive(PartialEq, Debug)]
enum Step {
    Local,
    Global((Vec3, Quat, Vec3)),
    Error,
}
/*impl<'a> MovePixil<'a> {
    pub fn start(member: &'a Member<PositionPixil>) -> Self {
        Self {
            member,
            step: Step::None,
            lock: None,
        }
    }
    pub fn set_rotation_local(mut self, angles: Quat) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.rotation = angles,
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_rotation_local(angles)
                }
            }
            Step::Global(..) => self.sync_local_from_global().set_rotation_local(angles),
            Step::None => {
                self.step = Step::Local;
                self.set_rotation_local(angles)
            }
            Step::Error => self,
        }
    }
    pub fn rotate_around_local(mut self, angle: f32,axis: Vec3) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.rotation = v.rotation.add(Quat::from_axis_angle(axis, angle)),
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.rotate_around_local(angle,axis)
                }
            }
            Step::Global(..) => self.sync_local_from_global().rotate_around_local(angle,axis),
            Step::None => {
                self.step = Step::Local;
                self.rotate_around_local(angle,axis)
            }
            Step::Error => self,
        }
    }
    pub fn set_rotation_global(mut self, angles:Quat) -> Self {
        match self.step {
            Step::Global(mut d) => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            d.1 = angles
                        },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_rotation_local(angles)
                }
            }
            Step::Local => self.sync_local_from_global().set_rotation_local(angles),
            Step::None => {
                self.step = Step::Global(self.);
                self.set_rotation_local(angles)
            }
            Step::Error => self,
        }
    }
    pub fn rotate_around_global(mut self, angle: f32,axis: Vec3) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.rotation = v.rotation.add(Quat::from_axis_angle(axis, angle)),
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.rotate_around_global(angle,axis)
                }
            }
            Step::Local => self.sync_global_from_local().rotate_around_global(angle,axis),
            Step::None => {
                self.step = Step::Global;
                self.rotate_around_global(angle,axis)
            }
            Step::Error => self,
        }
    }
    pub fn set_transition_local(mut self, x: f32, y: f32,z:f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.position = Vec3 { x, y,z },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_transition_local(x, y,z)
                }
            }
            Step::Global => self.sync_local_from_global().set_transition_local(x, y,z),
            Step::None => {
                self.step = Step::Local;
                self.set_transition_local(x, y,z)
            }
            Step::Error => self,
        }
    }
    pub fn transition_local(mut self, x: f32, y: f32, z:f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            v.position.x += x;
                            v.position.y += y;
                            v.position.z += z;
                        }
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.transition_local(x, y,z)
                }
            }
            Step::Global => self.sync_local_from_global().transition_local(x, y,z),
            Step::None => {
                self.step = Step::Local;
                self.transition_local(x, y,z)
            }
            Step::Error => self,
        }
    }
    pub fn set_transition_global(mut self, x: f32, y: f32, z:f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.position = Vec3 { x, y, z },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_transition_global(x, y , z)
                }
            }
            Step::Local => self.sync_global_from_local().set_transition_global(x, y, z),
            Step::None => {
                self.step = Step::Global;
                self.set_transition_global(x, y, z)
            }
            Step::Error => self,
        }
    }
    pub fn transition_global(mut self, x: f32, y: f32, z:f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            v.global_position.x += x;
                            v.global_position.y += y;
                            v.global_position.y += y;
                        }
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.transition_global(x, y)
                }
            }
            Step::Local => self.sync_global_from_local().transition_global(x, y),
            Step::None => {
                self.step = Step::Global;
                self.transition_global(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn set_scale_local(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.local_scale = Vec2 { x, y },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_scale_local(x, y)
                }
            }
            Step::Global => self.sync_local_from_global().set_scale_local(x, y),
            Step::None => {
                self.step = Step::Local;
                self.set_scale_local(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn scale_local(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            v.local_scale.x += x;
                            v.local_scale.y += y;
                        }
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.scale_local(x, y)
                }
            }
            Step::Global => self.sync_local_from_global().scale_local(x, y),
            Step::None => {
                self.step = Step::Local;
                self.scale_local(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn set_scale_global(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.global_scale = Vec2 { x, y },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_scale_global(x, y)
                }
            }
            Step::Local => self.sync_global_from_local().set_scale_global(x, y),
            Step::None => {
                self.step = Step::Global;
                self.set_scale_global(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn scale_global(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            v.global_scale.x += x;
                            v.global_scale.y += y;
                        }
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.scale_global(x, y)
                }
            }
            Step::Local => self.sync_global_from_local().scale_global(x, y),
            Step::None => {
                self.step = Step::Global;
                self.scale_global(x, y)
            }
            Step::Error => self,
        }
    }

    pub fn finish(mut self) -> Result<(), ExpiredValue> {
        if let Step::Global(_) = self.step {
            if self.sync_local_from_global().step == Step::None {
                return Ok(());
            }
        } else {
            if self.step == Step::Local {
                if self.sync_global_from_local().step == Step::None {
                    return Ok(());
                }
            }
        }
        Err(ExpiredValue {})
    }

    fn to_global(mut self) -> Self {
        match &mut self.lock {
            None => { self.lock = Some(self.member.dry_f_write()); return self.to_global() },
            Some(t) => {
                match &*t {
                    Reference::Some(v) => {
                        self.step == Step::Global(v.global.to_scale_rotation_translation());
                    }
                    Reference::Expired => {
                        self.step == Step::Error;
                    }
                }
            }
        }
        self
    }
    fn sync_global_from_local(mut self) -> Self {
        self.step = Step::None;
        self.lock = None;
        let mut finished: bool = false;
        self.member.f_write(|mut v| {
            if let Reference::Some(v) = &mut *v {
                if let Some(t) = self.member.get_parent() {
                    if let Reference::Some(parent) = (&*t.f_read()) {
                        let sin_r = parent.global_rotation.sin();
                        let cos_r = parent.global_rotation.cos();

                        let rotated_local = Vec2 {
                            x: v.local_position.x * cos_r - v.local_position.y * sin_r,
                            y: v.local_position.x * sin_r + v.local_position.y * cos_r,
                        };

                        let scaled_local = Vec2 {
                            x: rotated_local.x * parent.global_scale.x,
                            y: rotated_local.y * parent.global_scale.y,
                        };

                        v.global_position = Vec2 {
                            x: parent.global_position.x + scaled_local.x,
                            y: parent.global_position.y + scaled_local.y,
                        };

                        v.global_rotation = parent.global_rotation + v.local_rotation;

                        v.global_scale = Vec2 {
                            x: parent.global_scale.x * v.local_scale.x,
                            y: parent.global_scale.y * v.local_scale.y,
                        };
                    } else {
                        v.global_position = v.local_position;
                        v.global_rotation = v.local_rotation;
                        v.global_scale = v.local_scale;
                    }
                } else {
                    v.global_position = v.local_position;
                    v.global_rotation = v.local_rotation;
                    v.global_scale = v.local_scale;
                };
                v.dirty = true;
                finished = true;
            }
        });
        if !finished {
            Self {
                member: self.member,
                step: Step::Error,
                lock: None,
            }
        } else {
            self
        }
    }
    fn sync_local_from_global(mut self) -> Self {
        self.step = Step::None;
        self.lock = None;
        let mut finished: bool = false;
        self.member.f_write(|mut v| {
            if let Reference::Some(v) = &mut *v {
                if let Some(t) = self.member.get_parent() {
                    if let Reference::Some(parent) = (&*t.f_read()) {
                        let inv_scale = Vec2 {
                            x: 1.0 / parent.global_scale.x,
                            y: 1.0 / parent.global_scale.y,
                        };

                        let delta = Vec2 {
                            x: v.global_position.x - parent.global_position.x,
                            y: v.global_position.y - parent.global_position.y,
                        };

                        let sin_r = parent.global_rotation.sin();
                        let cos_r = parent.global_rotation.cos();

                        let local_x = cos_r * delta.x + sin_r * delta.y;
                        let local_y = -sin_r * delta.x + cos_r * delta.y;

                        v.local_position = Vec2 {
                            x: local_x * inv_scale.x,
                            y: local_y * inv_scale.y,
                        };

                        v.local_rotation = v.global_rotation - parent.global_rotation;

                        v.local_scale = Vec2 {
                            x: v.global_scale.x / parent.global_scale.x,
                            y: v.global_scale.y / parent.global_scale.y,
                        };
                    } else {
                        v.local_position = v.global_position;
                        v.local_rotation = v.global_rotation;
                        v.local_scale = v.global_scale;
                    }
                } else {
                    v.local_position = v.global_position;
                    v.local_rotation = v.global_rotation;
                    v.local_scale = v.global_scale;
                };
                finished = true;
                v.dirty = true;
            }
        });
        if !finished {
            Self {
                member: self.member,
                step: Step::Error,
                lock: None,
            }
        } else {
            self
        }
    }
}*/
impl<'a> MovePixil<'a> {
    pub fn start(member: &'a Member<PositionPixil>) -> Self {
        let mut lock = member.dry_f_write();
        if lock.is_expired() {
            Self {
                member,
                step: Step::Local,
                lock: Lock::Lock(member.dry_f_write()),
            }
        }else {
            Self {
                member,
                step: Step::Error,
                lock: Lock::NotLock,
            }
        }

    }
    pub fn finish(mut self) -> Result<(), ExpiredValue> {
        match &self.step {
            Step::Local => {self.to_global()}
            Step::Global(..) => {self.to_local()}
            _=>{}
        }
        self.lock.unwrap().dirty = true;
        self.lock.drop();
        self.member.shared_behavior();
        if self.step == Step::Error {
            Ok(())
        } else {
            Err(ExpiredValue {})
        }
    }

    pub fn set_rotation_local(mut self, angles: Quat) -> Self {
        match self.step {
            Step::Local => {
                self.lock.unwrap().rotation = angles;
                self
            }
                Step::Global(..) => {
                    self.to_local();
                    self.set_rotation_local(angles)
                },
                Step::Error => self,
        }
    }
    pub fn set_rotation_global(mut self, angles: Quat) -> Self {
        match self.step {
            Step::Global((_, mut r,_)) => {
                r = angles;
                self
            }
            Step::Local => {
                self.to_global();
                self.set_rotation_global(angles)
            },
            Step::Error => self,
        }
    }
    pub fn rotate_around_local(mut self, angle: f32,axis: Vec3) -> Self {
        match self.step {
            Step::Local => {
                let mut r = &self.lock.unwrap().rotation;
                r = &r.add(Quat::from_axis_angle(axis, angle));
                self
            }
            Step::Global(..) => {
                self.to_local();
                self.rotate_around_local(angle,axis)
            },
            Step::Error => self,
        }
    }
    pub fn rotate_around_global(mut self, angle: f32,axis: Vec3) -> Self {
        match self.step {
            Step::Global((_, mut r,_)) => {
                r = r.add(Quat::from_axis_angle(axis, angle));
                self
            }
            Step::Local => {
                self.to_global();
                self.rotate_around_global(angle,axis)
            },
            Step::Error => self,
        }
    }
    pub fn look_at(mut self, target: Vec3) -> Self {
        match self.step {
            Step::Global((_, mut r,p)) => {
                let forward = (target - p).normalize();

                r = Mat4::from_cols(
                    Vec4::X,
                    Vec4::Y,
                    forward.extend(0.0),
                    Vec4::W
                ).to_scale_rotation_translation().1;

                self
            }
            Step::Local => {
                self.to_global();
                self.look_at(target)
            },
            Step::Error => self,
        }
    }
    pub fn look_at_member(mut self, target: &Member<PositionPixil>) -> Self {
        let target = match &*target.f_read() {
            Reference::Some(v) => {
                v.global.to_scale_rotation_translation().2
            }
            Reference::Expired => {return self}
        };
        self.look_at(target)
    }

    pub fn set_transition_local(mut self, position: Vec3) -> Self {
        match self.step {
            Step::Local => {
                self.lock.unwrap().position = position;
                self
            }
            Step::Global(..) => {
                self.to_local();
                self.set_transition_local(position)
            },
            Step::Error => self,
        }
    }
    pub fn set_transition_global(mut self, position: Vec3) -> Self {
        match self.step {
            Step::Global((_,_,mut p)) => {
                p = position;
                self
            }
            Step::Local => {
                self.to_local();
                self.set_transition_global(position)
            },
            Step::Error => self,
        }
    }
    pub fn transition_local(mut self, position: Vec3) -> Self {
        match self.step {
            Step::Local => {
                self.lock.unwrap().position += position;
                self
            }
            Step::Global(..) => {
                self.to_local();
                self.transition_local(position)
            },
            Step::Error => self,
        }
    }
    pub fn transition_global(mut self, position: Vec3) -> Self {
        match self.step {
            Step::Global((_,_,mut p)) => {
                p += position;
                self
            }
            Step::Local => {
                self.to_local();
                self.transition_global(position)
            },
            Step::Error => self,
        }
    }

    pub fn set_scale_local(mut self, scale: Vec3) -> Self {
        match self.step {
            Step::Local => {
                self.lock.unwrap().scale = scale;
                self
            }
            Step::Global(..) => {
                self.to_local();
                self.set_scale_local(scale)
            },
            Step::Error => self,
        }
    }
    pub fn set_scale_global(mut self, scale: Vec3) -> Self {
        match self.step {
            Step::Global((mut s,_,_)) => {
                s = scale;
                self
            }
            Step::Local => {
                self.to_local();
                self.set_scale_global(scale)
            },
            Step::Error => self,
        }
    }
    pub fn scale_local(mut self, scale: Vec3) -> Self {
        match self.step {
            Step::Local => {
                self.lock.unwrap().scale += scale;
                self
            }
            Step::Global(..) => {
                self.to_local();
                self.scale_local(scale)
            },
            Step::Error => self,
        }
    }
    pub fn scale_global(mut self, scale: Vec3) -> Self {
        match self.step {
            Step::Global((mut s,_,_)) => {
                s += scale;
                self
            }
            Step::Local => {
                self.to_local();
                self.scale_global(scale)
            },
            Step::Error => self,
        }
    }

    fn to_global(&mut self) {
        match &self.step {
            Step::Global(_) => {},
            Step::Error => {},
            Step::Local => match &mut *self.lock {
                Reference::Some(s) => {
                    let parents_transform = match self.member.get_parent() {
                        None => {
                            self.step = Step::Global(s.global.to_scale_rotation_translation());
                            return;
                        }
                        Some(v) => match &*v.f_read() {
                            Reference::Some(v) => v.global,
                            Reference::Expired => {
                                self.step = Step::Global(s.global.to_scale_rotation_translation());
                                return;
                            }
                        },
                    };
                    Step::Global(
                        (parents_transform
                            * Mat4::from_scale_rotation_translation(
                                s.scale, s.rotation, s.position,
                            ))
                        .to_scale_rotation_translation(),
                    );
                    return;
                }
                Reference::Expired => {
                    self.step = Step::Error;
                    return;
                }
            },
        }
    }
    fn to_local(&mut self)  {
        match &self.step {
            Step::Local => {},
            Step::Error => {},
            Step::Global((scale, rotation, position)) => match &mut *self.lock {
                Reference::Some(s) => {
                    let inv_parents_transform = match self.member.get_parent() {
                        None => {
                            s.scale = scale.clone();
                            s.rotation = rotation.clone();
                            s.position = position.clone();
                            s.global = Mat4::from_scale_rotation_translation(*scale, *rotation, *position);
                            self.step = Step::Local;
                            return;
                        }
                        Some(v) => match &*v.f_read() {
                            Reference::Some(v) => v.global.inverse(),
                            Reference::Expired => {
                                s.scale = scale.clone();
                                s.rotation = rotation.clone();
                                s.position = position.clone();
                                s.global = Mat4::from_scale_rotation_translation(*scale, *rotation, *position);
                                self.step = Step::Local;
                                return;
                            }
                        },
                    };
                    let (scale,rotation,translation) = (inv_parents_transform * s.global).to_scale_rotation_translation();
                    s.scale = scale;
                    s.rotation = rotation;
                    s.position = translation;
                    self.step = Step::Local;
                    return;
                }
                Reference::Expired => {
                    self.step = Step::Error;
                    return;
                }
            },
        }
    }
}
