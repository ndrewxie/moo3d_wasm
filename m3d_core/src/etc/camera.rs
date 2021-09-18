use crate::rendermath::{Matrix, Point3D, RenderMatrices};

pub const BLOCKS_PER_WIDTH: usize = 7;
pub const UNITS_PER_BLOCK: usize = 4;
const UNITS_PER_WIDTH: usize = BLOCKS_PER_WIDTH * UNITS_PER_BLOCK;

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
    pub scale: usize,
    pub center_x: f32,
    pub center_y: f32,
}
pub struct CameraCache {
    pub scale: Option<Matrix>,
    pub reverse_frustum: Option<Matrix>,
    pub center: Option<Matrix>,
    pub to_screen_space: Option<Matrix>,
}

impl Camera {
    pub fn new(
        position: Point3D,
        target: (f32, f32),
        fov_horizontal: f32,
        render_distance: usize,
        width: usize,
        height: usize,
    ) -> Self {
        let scale = width / UNITS_PER_WIDTH;
        let near = (UNITS_PER_WIDTH as f32 / 2.0) / (fov_horizontal / 2.0).tan();

        Self {
            data: CameraData {
                position,
                target,
                near,
                far: (render_distance * UNITS_PER_BLOCK) as f32 + near,
                scale,
                center_x: width as f32 / 2.0,
                center_y: height as f32 / 2.0,
            },
            cache: CameraCache::new(),
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
    pub fn scale(&self) -> usize {
        self.data.scale
    }
    pub fn near(&self) -> f32 {
        self.data.near
    }
    pub fn far(&self) -> f32 {
        self.data.far
    }
    pub fn screen_near(&self) -> f32 {
        self.data.scale as f32 * self.data.near
    }
    pub fn screen_far(&self) -> f32 {
        self.data.scale as f32 * self.data.far
    }
    pub fn reverse_frustum(point: &Point3D, cache: &mut CameraCache, data: &CameraData) -> Point3D {
        point.transform(CameraCache::reverse_frustum(
            &mut cache.reverse_frustum,
            data,
        ))
    }
    pub fn in_frustum(
        point: &Point3D,
        cache: &mut CameraCache,
        data: &CameraData,
    ) -> Option<Point3D> {
        let transformed = Self::reverse_frustum(point, cache, data);

        let z_factor = (transformed.get(2) / data.near).abs();
        let x_cutoff = (z_factor * data.center_x / (data.scale as f32)).ceil();
        let y_cutoff = (z_factor * data.center_y / (data.scale as f32)).ceil();

        if transformed.get(0).abs() > x_cutoff {
            None
        } else if transformed.get(1).abs() > y_cutoff {
            None
        } else if transformed.get(2).abs() > data.far || transformed.get(2).abs() < data.near {
            None
        } else {
            Some(transformed)
        }
    }
}
impl CameraCache {
    pub fn new() -> Self {
        Self {
            center: None,
            to_screen_space: None,
            scale: None,
            reverse_frustum: None,
        }
    }
    pub fn invalidate(&mut self) {
        self.center = None;
        self.to_screen_space = None;
        self.scale = None;
        self.reverse_frustum = None;
    }
    pub fn to_screen_space<'a>(
        to_screen_space: &'a mut Option<Matrix>,
        camera_data: &CameraData,
    ) -> &'a Matrix {
        if to_screen_space.is_none() {
            *to_screen_space = Some(
                RenderMatrices::translation(camera_data.center_x, camera_data.center_y, 0.0)
                    .matrix_mul(&RenderMatrices::scale(
                        camera_data.scale as f32,
                        camera_data.scale as f32,
                        1.0,
                    ))
                    .matrix_mul(&RenderMatrices::projection(
                        camera_data.near,
                        camera_data.far,
                    )),
            );
        }
        to_screen_space.as_ref().unwrap()
    }
    pub fn scale<'a>(scale: &'a mut Option<Matrix>, camera_data: &CameraData) -> &'a Matrix {
        if scale.is_none() {
            *scale = Some(RenderMatrices::scale(
                camera_data.scale as f32,
                camera_data.scale as f32,
                camera_data.scale as f32,
            ));
        }
        scale.as_ref().unwrap()
    }
    pub fn reverse_frustum<'a>(
        reverse_frustum: &'a mut Option<Matrix>,
        camera_data: &CameraData,
    ) -> &'a Matrix {
        if reverse_frustum.is_none() {
            *reverse_frustum = Some(
                RenderMatrices::rotation_x(camera_data.target.1)
                    .matrix_mul(&RenderMatrices::rotation_y(camera_data.target.0))
                    .matrix_mul(&RenderMatrices::translation(
                        -camera_data.position.position.get(0),
                        -camera_data.position.position.get(1),
                        -camera_data.position.position.get(2),
                    )),
            );
        }
        reverse_frustum.as_ref().unwrap()
    }
}
