use std::error::Error;

use egui_tetra::egui;
use egui_tetra::egui::CtxRef;
use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Camera, Color};
use tetra::input::MouseButton;
use tetra::Context;

use crate::camera_handling::camera_state::CameraState;
use crate::graph::edge::{
    PULL_FORCE_FORCE_AT_TWICE_DISTANCE, PULL_FORCE_MIN_DISTANCE, PUSH_FORCE_DISTANCE,
    PUSH_FORCE_FORCE,
};
use crate::graph::gravity::{PullForceConfig, PushForceConfig};
use crate::graph::{Graph, GraphOnCanvas};
use crate::input::input_state::{InputState, StateData};
use crate::step_algorithms::AlgorithmResult;
use crate::ui::ui_drawing::create_ui;
use crate::ui::ui_drawing::UiData;

pub const SCREEN_WIDTH: f32 = 1280.;
pub const SCREEN_HEIGHT: f32 = 800.;

pub struct GameState {
    pub graph: Graph,
    // This is problematic to make nonpublic.
    pub input_state: InputState,
    camera: Camera,

    scaler: ScreenScaler,

    pub ui_data: UiData,

    algorithm: Option<AlgorithmResult>,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameState {
        GameState {
            graph: Graph::new(),
            input_state: InputState::Move(StateData::default()),
            camera: Camera::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            scaler: ScreenScaler::with_window_size(
                ctx,
                SCREEN_WIDTH as i32,
                SCREEN_HEIGHT as i32,
                ScalingMode::ShowAllPixelPerfect,
            )
            .unwrap(),
            ui_data: UiData::new(),
            algorithm: None,
        }
    }

    pub fn add_algorithm(&mut self, algorithm_res: AlgorithmResult) {
        self.algorithm = Some(algorithm_res);
    }

    pub fn push_conf(&self) -> PushForceConfig {
        self.ui_data.push_conf
    }

    pub fn pull_conf(&self) -> PullForceConfig {
        self.ui_data.pull_conf
    }
}

impl egui_tetra::State<Box<dyn Error>> for GameState {
    fn ui(&mut self, ctx: &mut Context, egui_ctx: &CtxRef) -> Result<(), Box<dyn Error>> {
        create_ui(self, ctx, egui_ctx);

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context, egui_ctx: &CtxRef) -> Result<(), Box<dyn Error>> {
        self.graph
            .update(ctx, egui_ctx, &self.push_conf(), &self.pull_conf());

        if let Some(alg) = &mut self.algorithm {
            alg.update(ctx, &mut self.graph);
        }

        self.camera.update_camera_transformation(ctx)
    }

    fn draw(&mut self, ctx: &mut Context, egui_ctx: &egui::CtxRef) -> Result<(), Box<dyn Error>> {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::set_transform_matrix(ctx, self.camera.as_matrix());

        self.graph
            .draw(self.camera.mouse_position(ctx), ctx, egui_ctx);

        graphics::reset_transform_matrix(ctx);

        self.scaler.draw(ctx);

        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut Context,
        _egui_ctx: &CtxRef,
        event: tetra::Event,
    ) -> Result<(), Box<dyn Error>> {
        if let tetra::Event::MouseMoved { .. } = &event {
            self.input_state
                .on_mouse_drag(ctx, &mut self.graph, self.camera.mouse_position(ctx));
        }

        if let tetra::Event::MouseButtonPressed {
            button: MouseButton::Left,
        } = &event
        {
            self.input_state
                .on_left_click(ctx, &mut self.graph, self.camera.mouse_position(ctx));
        }

        self.camera.handle_camera_events(event)
    }
}
