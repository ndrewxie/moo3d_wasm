#[repr(C)]
#[derive(Debug)]
pub struct Vector {
    pub elements: Vec<f32>,
    pub dims: usize,
}
#[repr(C)]
#[derive(Debug)]
pub struct Matrix {
    pub width: usize,
    pub height: usize,
    pub elements: Vec<f32>,
}
#[repr(C)]
#[derive(Debug)]
pub struct Point3D {
    pub position: Vector,
}

impl Vector {
    pub fn new(dims: usize) -> Self {
        Self {
            elements: Vec::with_capacity(dims),
            dims,
        }
    }
    pub fn with_data(input: Vec<f32>) -> Self {
        Self {
            dims: input.len(),
            elements: input,
        }
    }
    pub fn with_fill(dims: usize, fill: f32) -> Self {
        Self {
            elements: vec![fill; dims],
            dims,
        }
    }
    #[inline(always)]
    pub fn set(&mut self, indx: usize, val: f32) {
        unsafe {
            *self.elements.get_unchecked_mut(indx) = val;
        }
    }
    #[inline(always)]
    pub fn get(&self, indx: usize) -> f32 {
        unsafe {
            *self.elements.get_unchecked(indx)
        }
    }
    pub fn norm(&self) -> f32 {
        let mut acc: f32 = 0.0;
        for element in self.elements.iter() {
            acc += element * element;
        }
        acc.sqrt()
    }
    pub fn normalize_inplace(&mut self) {
        self.scalar_mul_inplace(1.0 / self.norm());
    }
    pub fn normalize(&self) -> Self {
        let factor = 1.0 / self.norm();
        let mut to_return = Self::with_fill(self.dims, 0.0);
        for (i, element) in self.elements.iter().enumerate() {
            to_return.set(i, element * factor);
        }
        to_return
    }
    pub fn minus(&self, rhs: &Self) -> Self {
        assert!(self.dims >= 3 && rhs.dims >= 3);
        Self::with_data(vec![
            self.get(0) - rhs.get(0),
            self.get(1) - rhs.get(1),
            self.get(2) - rhs.get(2),
        ])
    }
    pub fn cross(&self, rhs: &Self) -> Self {
        assert!(self.dims >= 3 && rhs.dims >= 3);
        Self::with_data(vec![
            self.get(1) * rhs.get(2) - self.get(2) * rhs.get(1),
            self.get(2) * rhs.get(0) - self.get(0) * rhs.get(2),
            self.get(0) * rhs.get(1) - self.get(1) * rhs.get(0),
        ])
    }
    pub fn scalar_mul_inplace(&mut self, factor: f32) {
        for element in self.elements.iter_mut() {
            *element *= factor;
        }
    }
    pub fn scalar_mul(&self, factor: f32) -> Self {
        let mut to_return = Self::with_fill(self.dims, 0.0);
        for (i, element) in self.elements.iter().enumerate() {
            to_return.set(i, element * factor);
        }
        to_return
    }
    pub fn homo_to_euc_inplace(&mut self) {
        assert_eq!(self.dims, 4);
        let w = self.get(3);
        self.set(0, self.get(0) / w);
        self.set(1, self.get(1) / w);
        self.set(2, self.get(2) / w);
        self.set(3, self.get(3) / w);
    }
    pub fn homo_to_euc(&self) -> Self {
        assert_eq!(self.dims, 4);
        let w = self.get(3);
        let mut to_return = Self::with_fill(self.dims - 1, 0.0);
        to_return.set(0, self.get(0) / w);
        to_return.set(1, self.get(1) / w);
        to_return.set(2, self.get(2) / w);
        to_return.set(3, self.get(3) / w);
        to_return
    }
}
impl Matrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            elements: Vec::with_capacity(width * height),
        }
    }
    pub fn with_fill(width: usize, height: usize, fill: f32) -> Self {
        Self {
            width,
            height,
            elements: vec![fill; width * height],
        }
    }
    pub fn with_flat_data(width: usize, height: usize, elements: Vec<f32>) -> Self {
        Self {
            width,
            height,
            elements,
        }
    }
    pub fn with_2d_data(data: &Vec<Vec<f32>>) -> Self {
        let width = data[0].len();
        let height = data.len();
        let mut elements = Vec::with_capacity(data.len() * data[0].len());
        for row in data.iter() {
            assert_eq!(row.len(), width);
            for value in row.iter() {
                elements.push(value.clone());
            }
        }
        Self {
            width,
            height,
            elements,
        }
    }
    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        unsafe { *self.elements.get_unchecked(y * self.width + x) }
    }
    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        unsafe {
            *self.elements.get_unchecked_mut(y * self.width + x) = val;
        }
    }
    pub fn vector_mul(&self, rhs: &Vector) -> Vector {
        assert_eq!(self.width, rhs.dims);
        let mut to_return = Vector::with_fill(self.height, 0.0);
        for yindx in 0..self.height {
            for xindx in 0..self.width {
                to_return.set(
                    yindx,
                    to_return.get(yindx) + rhs.get(xindx) * self.get(xindx, yindx),
                );
            }
        }
        to_return
    }
    pub fn matrix_mul(&self, rhs: &Matrix) -> Matrix {
        assert_eq!(self.width, rhs.height);
        let mut to_return = Self::with_fill(rhs.width, self.height, 0.0);
        for row in 0..self.height {
            for column in 0..rhs.width {
                let mut acc = 0.0;
                for indx in 0..self.width {
                    acc += self.get(indx, row) * rhs.get(column, indx);
                }
                to_return.set(column, row, acc);
            }
        }
        to_return
    }
}
impl Point3D {
    pub fn from_euc_coords(x: isize, y: isize, z: isize) -> Self {
        Self {
            position: Vector::with_data(vec![x as f32, y as f32, z as f32, 1.0]),
        }
    }
    // haha funni name
    pub fn from_homo_coords(x: isize, y: isize, z: isize, w: f32) -> Self {
        Self {
            position: Vector::with_data(vec![x as f32, y as f32, z as f32, w]),
        }
    }
    pub fn homo_to_euc(&mut self) {
        self.position.homo_to_euc_inplace();
    }

    #[inline(always)]
    pub fn get(&self, n: usize) -> f32 {
        self.position.get(n)
    }
    #[inline(always)]
    pub fn set(&mut self, n: usize, val: f32) {
        self.position.set(n, val)
    }

    #[inline(always)]
    pub fn x_coord(&self) -> isize {
        (self.get(0) + 0.5) as isize
    }
    #[inline(always)]
    pub fn y_coord(&self) -> isize {
        (self.get(1) + 0.5) as isize
    }
    #[inline(always)]
    pub fn z_coord(&self) -> isize {
        (self.get(2) + 0.5) as isize
    }
    #[inline(always)]
    pub fn set_x_coord(&mut self, input: isize) {
        self.set(0, input as f32)
    }
    #[inline(always)]
    pub fn set_y_coord(&mut self, input: isize) {
        self.set(1, input as f32)
    }
    #[inline(always)]
    pub fn set_z_coord(&mut self, input: isize) {
        self.set(2, input as f32)
    }
    #[inline(always)]
    pub fn z_coord_float(&self) -> f32 {
        self.get(2)
    }
    #[inline(always)]
    pub fn w_coord(&self) -> f32 {
        self.get(3)
    }
}

pub struct RenderMatrices {}
impl RenderMatrices {
    pub fn bundle_points(points: &[&Point3D]) -> Matrix {
        let mut to_return = Vec::with_capacity(4 * points.len());
        for indx in 0..4 {
            for elem in points.iter() {
                to_return.push(elem.position.get(indx));
            }
        }
        Matrix::with_flat_data(points.len(), 4, to_return)
    }
    pub fn split_points(input: &Matrix) -> Vec<Point3D> {
        let mut to_return = Vec::with_capacity(input.width);
        for indx in 0..input.width {
            let x = input.get(indx, 0);
            let y = input.get(indx, 1);
            let z = input.get(indx, 2);
            let w = input.get(indx, 3);

            let mut to_push = Point3D::from_homo_coords(x as isize, y as isize, z as isize, w);
            to_push.homo_to_euc();
            to_return.push(to_push);
        }
        to_return
    }
    pub fn compose_transformations(input: &[&Matrix]) -> Matrix {
        assert!(input.len() >= 2);
        let mut to_return = input[1].matrix_mul(input[0]);
        for indx in 2..input.len() {
            to_return = input[indx].matrix_mul(&to_return);
        }
        to_return
    }
    pub fn projection(near: isize, far: isize) -> Matrix {
        let n = near as f32;
        let f = far as f32;
        let a = f / (f - n);
        //let b = -f * n / (f - n);
        let b = -a * n;

        let mut to_return = Matrix::with_fill(4, 4, 0.0);
        to_return.set(0, 0, n);
        to_return.set(1, 1, n);
        to_return.set(2, 2, a);
        to_return.set(3, 2, b);
        to_return.set(2, 3, 1.0);

        to_return
    }
    pub fn rotation_x(theta: f32) -> Matrix {
        let cos = theta.cos();
        let sin = theta.sin();

        let mut to_return = Matrix::with_fill(4, 4, 0.0);
        to_return.set(0, 0, 1.0);
        to_return.set(1, 1, cos);
        to_return.set(2, 1, sin);
        to_return.set(1, 2, -sin);
        to_return.set(2, 2, cos);
        to_return.set(3, 3, 1.0);

        to_return
    }
    pub fn rotation_y(theta: f32) -> Matrix {
        let cos = theta.cos();
        let sin = theta.sin();

        let mut to_return = Matrix::with_fill(4, 4, 0.0);
        to_return.set(0, 0, cos);
        to_return.set(2, 0, -sin);
        to_return.set(1, 1, 1.0);
        to_return.set(0, 2, sin);
        to_return.set(2, 2, cos);
        to_return.set(3, 3, 1.0);

        to_return
    }
    pub fn rotation_z(theta: f32) -> Matrix {
        let cos = theta.cos();
        let sin = theta.sin();

        let mut to_return = Matrix::with_fill(4, 4, 0.0);
        to_return.set(0, 0, cos);
        to_return.set(1, 0, -sin);
        to_return.set(0, 1, sin);
        to_return.set(1, 1, cos);
        to_return.set(2, 2, 1.0);
        to_return.set(3, 3, 1.0);

        to_return
    }
    pub fn rotation_3d(
        thetax: f32,
        thetay: f32,
        thetaz: f32,
        translation: Option<&(f32, f32, f32)>,
    ) -> Matrix {
        match translation {
            Some((tx, ty, tz)) => Self::compose_transformations(&[
                &Self::translation(-*tx, -*ty, -*tz),
                &Self::rotation_z(thetaz),
                &Self::rotation_y(thetay),
                &Self::rotation_x(thetax),
                &Self::translation(*tx, *ty, *tz),
            ]),
            None => Self::rotation_x(thetax)
                .matrix_mul(&Self::rotation_y(thetay).matrix_mul(&Self::rotation_z(thetaz))),
        }
    }
    pub fn translation(tx: f32, ty: f32, tz: f32) -> Matrix {
        let mut to_return = Matrix::with_fill(4, 4, 0.0);
        to_return.set(0, 0, 1.0);
        to_return.set(1, 1, 1.0);
        to_return.set(2, 2, 1.0);
        to_return.set(3, 3, 1.0);
        to_return.set(3, 0, tx);
        to_return.set(3, 1, ty);
        to_return.set(3, 2, tz);

        to_return
    }
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Matrix {
        let mut to_return = Matrix::with_fill(4, 4, 0.0);
        to_return.set(0, 0, sx);
        to_return.set(1, 1, sy);
        to_return.set(2, 2, sz);
        to_return.set(3, 3, 1.0);

        to_return
    }
    // Faster, specialized version of det, valid only if the top row (a, b, c) are all 1.0
    fn bary_det(d: f32, e: f32, f: f32, g: f32, h: f32, i: f32) -> f32 {
        e * (i - g) - f * (h - g) + d * (h - i)
    }
    // Input: p0, p1 are coords for (min_x, min_y)
    // Returns (u, v, w, du/dx, dv/dx, dw/dx, du/dy, dv/dy, dw/dy) for upper-left corner
    pub fn barycentric_params(
        p0: f32,
        p1: f32,
        v10: f32,
        v11: f32,
        v20: f32,
        v21: f32,
        v30: f32,
        v31: f32,
    ) -> (f32, f32, f32, f32, f32, f32, f32, f32, f32) {
        let denom = 1.0 / Self::bary_det(v10, v20, v30, v11, v21, v31);

        let u = Self::bary_det(p0, v20, v30, p1, v21, v31) * denom;
        let v = Self::bary_det(v10, p0, v30, v11, p1, v31) * denom;
        let w = 1.0 - u - v;

        let dudx = denom * (v21 - v31);
        let dvdx = denom * (v31 - v11);
        let dwdx = -dudx - dvdx;

        let dudy = denom * (v30 - v20);
        let dvdy = denom * (v10 - v30);
        let dwdy = -dudy - dvdy;

        (u, v, w, dudx, dvdx, dwdx, dudy, dvdy, dwdy)
    }
}
