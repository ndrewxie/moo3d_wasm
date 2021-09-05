use crate::rendering::rendermath::{Matrix, Point3D, RenderMatrices, Vector};

pub struct Camera {
    pub data: CameraData,
    pub cache: CameraCache,
}
// ugly as hell but whatever
pub struct CameraData {
    pub position: Point3D,
    pub target: (f32, f32), // rotation, inclination
    pub near: f32,
    pub far: f32,
    pub center_x: f32,
    pub center_y: f32,
}
pub struct CameraCache {
    pub view: Option<Matrix>,
    pub projection: Option<Matrix>,
    pub reverse: Option<Matrix>,
}

impl Camera {
    pub fn new(
        position: Point3D,
        target: (f32, f32),
        near: f32,
        far: f32,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            data: CameraData {
                position,
                target,
                near,
                far,
                center_x: width as f32 / 2.0,
                center_y: height as f32 / 2.0,
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

        self.cache.invalidate();
    }
    pub fn translate_look(&mut self, d_rotation: f32, d_inclination: f32) {
        self.data.target.0 += d_rotation;
        self.data.target.1 += d_inclination;

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
            let forward = Point3D::from_euc_coords_float(0.0, 0.0, 1.0)
                .transform(&RenderMatrices::rotation_x(camera_data.target.1))
                .transform(&RenderMatrices::rotation_y(camera_data.target.0));
            let right = Vector::with_data(vec![0.0, 1.0, 0.0]).cross(&forward.position);
            let up = forward.position.cross(&right);

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
                camera_data.center_x,
                camera_data.center_y,
                0.0,
            ));
        }
        reverse.as_ref().unwrap()
    }
}
