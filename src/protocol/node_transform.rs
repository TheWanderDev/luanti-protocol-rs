#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeTransform {
    basis: [[f32; 3]; 3],
    facedir: u8,
}

impl NodeTransform {
    const IDENTITY: Self = Self {
        basis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        facedir: 0,
    };

    #[must_use]
    pub fn from_param_type2(param_type2: u8, param2: u8) -> Self {
        match param_type2 {
            3 | 9 => Self::from_facedir((param2 & 0x1F) % 24),
            4 | 10 => {
                const WALLMOUNTED_FACEDIR: [u8; 8] = [20, 0, 17, 15, 8, 6, 21, 1];
                Self::from_facedir(WALLMOUNTED_FACEDIR[usize::from(param2 & 0x07)])
            }
            6 => Self::from_y_rotation(f32::from(param2 % 240) * 1.5),
            13 | 14 => Self::from_facedir(param2 & 0x03),
            12 => Self::from_y_rotation(f32::from((param2 & 0x1F) % 24) * 15.0),
            _ => Self::IDENTITY,
        }
    }

    #[must_use]
    pub fn from_y_quarters(turns: u8) -> Self {
        Self::from_y_rotation(-f32::from(turns % 4) * 90.0)
    }

    #[must_use]
    pub fn from_facedir(facedir: u8) -> Self {
        let facedir = facedir % 24;
        let mut basis = [[0.0; 3]; 3];

        for (axis, vector) in basis.iter_mut().enumerate() {
            let mut point = [0.0; 3];
            point[axis] = 1.0;
            rotate_facedir(&mut point, facedir);
            *vector = point;
        }

        Self { basis, facedir }
    }

    #[must_use]
    pub const fn facedir(self) -> u8 {
        self.facedir
    }

    #[must_use]
    pub fn transform_point(self, point: [f32; 3]) -> [f32; 3] {
        [
            self.basis[0][0] * point[0] + self.basis[1][0] * point[1] + self.basis[2][0] * point[2],
            self.basis[0][1] * point[0] + self.basis[1][1] * point[1] + self.basis[2][1] * point[2],
            self.basis[0][2] * point[0] + self.basis[1][2] * point[1] + self.basis[2][2] * point[2],
        ]
    }

    #[must_use]
    pub fn transform_bounds(self, min: [f32; 3], max: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let mut transformed_min = [f32::MAX; 3];
        let mut transformed_max = [f32::MIN; 3];

        for mask in 0..8 {
            let point = [
                if mask & 1 == 0 { min[0] } else { max[0] },
                if mask & 2 == 0 { min[1] } else { max[1] },
                if mask & 4 == 0 { min[2] } else { max[2] },
            ];
            let point = self.transform_point(point);

            for (axis, value) in point.into_iter().enumerate() {
                transformed_min[axis] = transformed_min[axis].min(value);
                transformed_max[axis] = transformed_max[axis].max(value);
            }
        }

        (transformed_min, transformed_max)
    }

    fn from_y_rotation(degrees: f32) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        Self {
            basis: [[cos, 0.0, sin], [0.0, 1.0, 0.0], [-sin, 0.0, cos]],
            facedir: 0,
        }
    }
}

fn rotate_facedir(point: &mut [f32; 3], facedir: u8) {
    let facedir = facedir % 24;

    for _ in 0..facedir % 4 {
        let x = point[0];
        point[0] = point[2];
        point[2] = -x;
    }

    let [x, y, z] = *point;
    *point = match facedir / 4 {
        1 => [x, -z, y],
        2 => [x, z, -y],
        3 => [y, -x, z],
        4 => [-y, x, z],
        5 => [-x, -y, z],
        _ => [x, y, z],
    };
}
