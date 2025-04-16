#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn angle_between(self, other: Vec2) -> f32 {
        let dot = self.x * other.x + self.y * other.y;
        let det = self.x * other.y - self.y * other.x;
        det.atan2(dot) // signed angle between -π and π
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len != 0.0 {
            Vec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Vec2 { x: 0.0, y: 0.0 }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Mat3 {
    pub m: [[f32; 3]; 3],
}
impl Mat3 {
    pub fn identity() -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn translate(v: Vec2) -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [v.x, v.y, 1.0]],
        }
    }

    pub fn rotate(angle_rad: f32) -> Self {
        let (sin, cos) = angle_rad.sin_cos();
        Self {
            m: [[cos, sin, 0.0], [-sin, cos, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn scale(s: Vec2) -> Self {
        Self {
            m: [[s.x, 0.0, 0.0], [0.0, s.y, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn from_scale_rotation_translation(scale: Vec2, rotation: f32, translation: Vec2) -> Self {
        Mat3::translate(translation)
            .multiply(&Mat3::rotate(rotation))
            .multiply(&Mat3::scale(scale))
    }

    pub fn multiply(&self, other: &Mat3) -> Self {
        let mut result = Mat3::identity();
        for i in 0..3 {
            for j in 0..3 {
                result.m[i][j] = self.m[0][j] * other.m[i][0]
                    + self.m[1][j] * other.m[i][1]
                    + self.m[2][j] * other.m[i][2];
            }
        }
        result
    }

    pub fn inverse(&self) -> Option<Mat3> {
        // Extract 2x2 rotation/scale components and translation
        let a = self.m[0][0];
        let b = self.m[1][0];
        let c = self.m[0][1];
        let d = self.m[1][1];
        let tx = self.m[2][0];
        let ty = self.m[2][1];

        // Calculate determinant of the upper 2x2 matrix
        let det = a * d - b * c;
        if det.abs() < f32::EPSILON {
            return None; // Matrix is not invertible
        }

        let inv_det = 1.0 / det;

        // Calculate inverse of 2x2 matrix
        let inv_a = d * inv_det;
        let inv_b = -b * inv_det;
        let inv_c = -c * inv_det;
        let inv_d = a * inv_det;

        // Calculate inverse translation
        let inv_tx = (b * ty - d * tx) * inv_det;
        let inv_ty = (c * tx - a * ty) * inv_det;

        Some(Mat3 {
            m: [
                [inv_a, inv_c, 0.0],   // Column 0
                [inv_b, inv_d, 0.0],   // Column 1
                [inv_tx, inv_ty, 1.0], // Column 2 (translation)
            ],
        })
    }

    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        Vec2 {
            x: self.m[0][0] * point.x + self.m[1][0] * point.y + self.m[2][0],
            y: self.m[0][1] * point.x + self.m[1][1] * point.y + self.m[2][1],
        }
    }

    pub fn to_mat4_4(&self) -> [[f32; 4]; 4] {
        [
            [self.m[0][0], self.m[0][1], 0.0, self.m[0][2]],
            [self.m[1][0], self.m[1][1], 0.0, self.m[1][2]],
            [0.0, 0.0, 1.0, 0.0],
            [self.m[2][0], self.m[2][1], 0.0, self.m[2][2]],
        ]
    }

    pub fn decompose(&self) -> (Vec2, f32, Vec2) {
        let x_axis = Vec2 {
            x: self.m[0][0],
            y: self.m[1][0],
        };
        let y_axis = Vec2 {
            x: self.m[0][1],
            y: self.m[1][1],
        };

        let scale_x = x_axis.length();
        let scale_y = y_axis.length();
        let scale = Vec2 {
            x: scale_x,
            y: scale_y,
        };

        let rotation = x_axis.angle_between(Vec2 { x: 1.0, y: 0.0 });

        // Translation
        let translation = Vec2 {
            x: self.m[0][2],
            y: self.m[1][2],
        };

        (scale, rotation, translation)
    }
}
