pub struct PixelIterator {
    width: usize,
    height: usize,

    pub x: usize,
    pub y: usize,

    pub u: f32,
    pub v: f32,
    pub w: f32,
    row_u: f32,
    row_v: f32,
    row_w: f32,
    barycentric_deltas: [f32; 6],
    inv_x_deltas: [Option<f32>; 3],

    pub pixel_offset: usize,
    pub offset: usize,
}

impl PixelIterator {
    pub fn new(
        width: usize,
        height: usize,
        x: usize,
        y: usize,
        barycentric_params: &(f32, f32, f32, f32, f32, f32, f32, f32, f32),
    ) -> Self {
        let pixel_offset: usize = y * width + x;
        Self {
            width,
            height,

            x,
            y,

            u: barycentric_params.0,
            v: barycentric_params.1,
            w: barycentric_params.2,
            row_u: barycentric_params.0,
            row_v: barycentric_params.1,
            row_w: barycentric_params.2,

            barycentric_deltas: [
                barycentric_params.3, // dudx
                barycentric_params.4, // dvdx
                barycentric_params.5, // dwdx
                barycentric_params.6, // dudy
                barycentric_params.7, // dvdy
                barycentric_params.8, // dwdy
            ],
            inv_x_deltas: [
                if barycentric_params.3.abs() < 0.0001 {
                    None
                } else {
                    Some(1.0 / barycentric_params.3)
                },
                if barycentric_params.4.abs() < 0.0001 {
                    None
                } else {
                    Some(1.0 / barycentric_params.4)
                },
                if barycentric_params.5.abs() < 0.0001 {
                    None
                } else {
                    Some(1.0 / barycentric_params.5)
                },
            ],

            pixel_offset,
            offset: 4 * pixel_offset,
        }
    }
    pub fn next_row(&mut self) {
        self.y += 1;
        self.pixel_offset += self.width;
        self.offset += 4 * self.width;

        self.u += self.barycentric_deltas[3];
        self.v += self.barycentric_deltas[4];
        self.w += self.barycentric_deltas[5];

        self.row_u += self.barycentric_deltas[3];
        self.row_v += self.barycentric_deltas[4];
        self.row_w += self.barycentric_deltas[5];
    }
    pub fn next_column(&mut self) {
        self.x += 1;
        self.pixel_offset += 1;
        self.offset += 4;

        self.u += self.barycentric_deltas[0];
        self.v += self.barycentric_deltas[1];
        self.w += self.barycentric_deltas[2];
    }
    pub fn set_x(&mut self, x: isize) {
        let dx = x - self.x as isize;
        let dx_f = dx as f32;

        self.x = x as usize;
        self.pixel_offset = (self.pixel_offset as isize + dx) as usize;
        self.offset = 4 * self.pixel_offset;

        self.u += dx_f * self.barycentric_deltas[0];
        self.v += dx_f * self.barycentric_deltas[1];
        self.w += dx_f * self.barycentric_deltas[2];
    }
    pub fn move_to(&mut self, x: usize, y: usize) {
        let dx = (x as isize - self.x as isize) as f32;
        let dy = (y as isize - self.y as isize) as f32;

        self.x = x;
        self.y = y;
        self.pixel_offset = y * self.width + x;
        self.offset = 4 * self.pixel_offset;

        self.u += dx * self.barycentric_deltas[0] + dy * self.barycentric_deltas[3];
        self.v += dx * self.barycentric_deltas[1] + dy * self.barycentric_deltas[4];
        self.w += dx * self.barycentric_deltas[2] + dy * self.barycentric_deltas[5];

        self.row_u += dy * self.barycentric_deltas[3];
        self.row_v += dy * self.barycentric_deltas[4];
        self.row_w += dy * self.barycentric_deltas[5];
    }
    // Solves for the range of x-coordinates (euclidean) to make points in the triangle
    // Call it once for each of the 3 barycentric coordinates (u, v, w)
    // a is the current row's barycentric coordinate, b is the RECIPROCAL of du/dx, or dv/dx, or
    // dw/dx
    fn solve_bary_range(lower: &mut f32, upper: &mut f32, a: f32, b: f32) {
        let calculated = -a * b;
        if b >= 0.0 {
            if *lower < calculated {
                *lower = calculated;
            }
        } else {
            if *upper > calculated {
                *upper = calculated;
            }
        }
    }
    pub fn solve_x_range(&self, min_x: isize, max_x: isize) -> [isize; 2] {
        let mut low = 0.0;
        let mut high = (max_x - min_x) as f32;
        if let Some(inv) = self.inv_x_deltas[0] {
            Self::solve_bary_range(&mut low, &mut high, self.row_u, inv)
        }
        if let Some(inv) = self.inv_x_deltas[1] {
            Self::solve_bary_range(&mut low, &mut high, self.row_v, inv)
        }
        if let Some(inv) = self.inv_x_deltas[2] {
            Self::solve_bary_range(&mut low, &mut high, self.row_w, inv)
        }

        [low as isize, (high + 0.5) as isize]
    }
}
