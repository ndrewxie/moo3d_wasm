use crate::rendering::rendermath::{Matrix, Point3D, RenderMatrices, Vector};

pub struct Camera {
    pub data: CameraData,
    pub cache: CameraCache,
}
// ugly as hell but whatever
pub struct CameraData {
    pub position: Point3D,
    pub target: Point3D,
    pub near: isize,
    pub far: isize,
}
pub struct CameraCache {
    pub view: Option<Matrix>,
    pub projection: Option<Matrix>,
    pub reverse: Option<Matrix>,
}

impl Camera {
    pub fn new(position: Point3D, target: Point3D, near: isize, far: isize) -> Self {
        Self {
            data: CameraData {
                position,
                target,
                near,
                far,
            },
            cache: CameraCache {
                view: None,
                projection: None,
                reverse: None,
            },
        }
    }
    pub fn translate(&mut self, dx: isize, dy: isize, dz: isize) {
        let data = &mut self.data;
        data.position.set(0, data.position.get(0) + dx as f32);
        data.position.set(1, data.position.get(1) + dy as f32);
        data.position.set(2, data.position.get(2) + dz as f32);

        data.target.set(0, data.target.get(0) + dx as f32);
        data.target.set(1, data.target.get(1) + dy as f32);
        data.target.set(2, data.target.get(2) + dz as f32);

        self.cache.invalidate();
    }
    pub fn translate_look(&mut self, dx: f32, dy: f32, dz: f32) {
        let data = &mut self.data;
        data.target.set(0, data.target.get(0) + dx);
        data.target.set(1, data.target.get(1) + dy);
        data.target.set(2, data.target.get(2) + dz);

        self.cache.invalidate();
    }
    pub fn look(&mut self, target: Point3D) {
        self.data.target = target;

        self.cache.invalidate();
    }
}
impl CameraCache {
    pub fn invalidate(&mut self) {
        self.view = None;
        self.projection = None;
        self.reverse = None;
    }
    pub fn projection<'a>(
        projection: &'a mut Option<Matrix>,
        camera_data: &CameraData,
    ) -> &'a Matrix {
        if projection.is_none() {
            *projection = Some(RenderMatrices::projection(
                camera_data.near,
                camera_data.far,
            ));
        }
        projection.as_ref().unwrap()
    }
    pub fn view<'a>(view: &'a mut Option<Matrix>, camera_data: &CameraData) -> &'a Matrix {
        if view.is_none() {
            let mut forward = camera_data
                .target
                .position
                .minus(&camera_data.position.position);
            forward.normalize_inplace();
            let right = Vector::with_data(vec![0.0, 1.0, 0.0]).cross(&forward);
            let up = forward.cross(&right);

            let coord_matrix = Matrix::with_2d_data(&vec![
                vec![right.get(0), up.get(0), forward.get(0), 0.0],
                vec![right.get(1), up.get(1), forward.get(1), 0.0],
                vec![right.get(2), up.get(2), forward.get(2), 0.0],
                vec![0.0, 0.0, 0.0, 1.0],
            ]);

            let translation_matrix = RenderMatrices::translation(
                -1.0 * camera_data.position.get(0),
                -1.0 * camera_data.position.get(1),
                -1.0 * camera_data.position.get(2),
            );

            *view = Some(coord_matrix.matrix_mul(&translation_matrix));
        }
        view.as_ref().unwrap()
    }
    pub fn reverse<'a>(reverse: &'a mut Option<Matrix>, camera_data: &CameraData) -> &'a Matrix {
        if reverse.is_none() {
            *reverse = Some(RenderMatrices::translation(
                camera_data.position.get(0),
                camera_data.position.get(1),
                0.0,
            ));
        }
        reverse.as_ref().unwrap()
    }
}
