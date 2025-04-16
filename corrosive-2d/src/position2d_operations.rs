use crate::comp::Position2D;
use crate::math2d::Vec2;
use corrosive_ecs_core::ecs_core::{Member, Reference};
use std::sync::RwLockWriteGuard;

pub struct Move2D<'a> {
    member: &'a Member<Position2D>,
    step: Step,
    lock: Option<RwLockWriteGuard<'a, Reference<Position2D>>>,
}
pub struct ExpiredValue {}
#[derive(Eq, PartialEq, Debug)]
pub enum Step {
    Local,
    Global,
    None,
    Error,
}
impl<'a> Move2D<'a> {
    pub fn start(member: &'a Member<Position2D>) -> Self {
        Self {
            member,
            step: Step::None,
            lock: None,
        }
    }
    pub fn set_rotation_local(mut self, angle: f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.local_rotation = angle,
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_rotation_local(angle)
                }
            }
            Step::Global => self.sync_local_from_global().set_rotation_local(angle),
            Step::None => {
                self.step = Step::Local;
                self.set_rotation_local(angle)
            }
            Step::Error => self,
        }
    }
    pub fn rotate_local(mut self, angle: f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.local_rotation += angle,
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.rotate_local(angle)
                }
            }
            Step::Global => self.sync_local_from_global().rotate_local(angle),
            Step::None => {
                self.step = Step::Local;
                self.rotate_local(angle)
            }
            Step::Error => self,
        }
    }
    pub fn set_rotation_global(mut self, angle: f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.global_rotation = angle,
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_rotation_global(angle)
                }
            }
            Step::Local => self.sync_global_from_local().set_rotation_global(angle),
            Step::None => {
                self.step = Step::Global;
                self.set_rotation_global(angle)
            }
            Step::Error => self,
        }
    }
    pub fn rotate_global(mut self, angle: f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.global_rotation += angle,
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.rotate_global(angle)
                }
            }
            Step::Local => self.sync_global_from_local().rotate_global(angle),
            Step::None => {
                self.step = Step::Global;
                self.rotate_global(angle)
            }
            Step::Error => self,
        }
    }
    pub fn set_transition_local(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.local_position = Vec2 { x, y },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Local,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_transition_local(x, y)
                }
            }
            Step::Global => self.sync_local_from_global().set_transition_local(x, y),
            Step::None => {
                self.step = Step::Local;
                self.set_transition_local(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn transition_local(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Local => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            v.local_position.x += x;
                            v.local_position.y += y;
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
                    self.transition_local(x, y)
                }
            }
            Step::Global => self.sync_local_from_global().transition_local(x, y),
            Step::None => {
                self.step = Step::Local;
                self.transition_local(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn set_transition_global(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => v.global_position = Vec2 { x, y },
                        Reference::Expired => {}
                    }
                    Self {
                        member: self.member,
                        step: Step::Global,
                        lock: Some(t),
                    }
                } else {
                    self.lock = Some(self.member.dry_f_write());
                    self.set_transition_global(x, y)
                }
            }
            Step::Local => self.sync_global_from_local().set_transition_global(x, y),
            Step::None => {
                self.step = Step::Global;
                self.set_transition_global(x, y)
            }
            Step::Error => self,
        }
    }
    pub fn transition_global(mut self, x: f32, y: f32) -> Self {
        match self.step {
            Step::Global => {
                if let Some(mut t) = self.lock {
                    match &mut *t {
                        Reference::Some(v) => {
                            v.global_position.x += x;
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
        if self.step == Step::Global {
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
}
