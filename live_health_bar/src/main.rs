use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::Instant,
};

use bytemuck::{NoUninit, cast_slice};
use eframe::{
    CreationContext, NativeOptions, WgpuConfiguration,
    egui::{self, Color32, Vec2},
    egui_wgpu::{self, WgpuSetup, WgpuSetupCreateNew},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
        BindingType, BlendState, ColorTargetState, ColorWrites, Device,
        Extent3d, Features, FragmentState, MultisampleState,
        PipelineCompilationOptions, PipelineLayoutDescriptor,
        PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
        Sampler, SamplerBindingType, SamplerDescriptor,
        ShaderModuleDescriptor, ShaderSource, ShaderStages,
        TextureDescriptor, TextureDimension, TextureFormat,
        TextureSampleType, TextureUsages, TextureViewDescriptor,
        TextureViewDimension, VertexState, util::DeviceExt,
        wgt::TextureDataOrder,
    },
};
use image::EncodableLayout;
use serde::{Deserialize, Serialize};

use crate::utils::Wrap;

pub mod utils;

fn main() {
    let config = Config::load();

    let mut wgpu_setup_create_new =
        WgpuSetupCreateNew::without_display_handle();

    wgpu_setup_create_new.device_descriptor = Arc::new(move |adapter| {
        let mut desc = (wgpu_setup_create_new.device_descriptor)(adapter);
        desc.required_features |= Features::IMMEDIATES;
        desc.required_limits.max_immediate_size = 8;
        desc
    });

    let wgpu_setup = WgpuSetup::CreateNew(wgpu_setup_create_new);
    let wgpu_options = WgpuConfiguration {
        wgpu_setup,
        ..WgpuConfiguration::default()
    };

    eframe::run_native(
        "LiveHealthBar",
        NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_transparent(true)
                .with_has_shadow(false)
                .with_inner_size(
                    config.last_inner_size.map(|it| it as f32),
                ),
            wgpu_options,
            ..NativeOptions::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(config, cc)))),
    )
    .unwrap();
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct HealthBarAssetsPath {
    health_empty: Option<PathBuf>,
    health_fading: Option<PathBuf>,
    health_low: Option<PathBuf>,
    health: Option<PathBuf>,
    shield_fading: Option<PathBuf>,
    shield: Option<PathBuf>,
}

impl HealthBarAssetsPath {
    const LAYER_COUNT: u32 = 6;

    fn load_file_or(
        path: Option<&Path>,
        or: &'static [u8],
    ) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        if let Some(path) = path {
            image::open(path).unwrap().into_rgba8()
        } else {
            image::load_from_memory(or).unwrap().into_rgba8()
        }
    }

    fn load(
        &self,
        device: &Device,
        queue: &Queue,
        label: Option<&str>,
        bind_group_layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> Result<(BindGroup, Vec2), &'static str> {
        let health_empty = Self::load_file_or(
            self.health_empty.as_deref(),
            include_bytes!("./bar_embedded/health_empty.png"),
        );
        let bar_dims = health_empty.dimensions();
        if bar_dims.0 == 0 || bar_dims.1 == 0 {
            return Err("empty image");
        }

        let size = (bar_dims.0 as usize) * (bar_dims.1 as usize) * 4;
        let mut data = vec![0_u8; size * Self::LAYER_COUNT as usize];
        data[0..size].copy_from_slice(health_empty.as_bytes());

        let mut idx = 1;
        macro_rules! load_file {
            ($name:ident) => {{
                let $name = Self::load_file_or(
                    self.$name.as_deref(),
                    include_bytes!(concat!(
                        "./bar_embedded/",
                        stringify!($name),
                        ".png"
                    )),
                );
                if $name.dimensions() != bar_dims {
                    return Err("images not same size");
                }
                data[(size * idx)..(size * (idx + 1))]
                    .copy_from_slice($name.as_bytes());
                idx += 1;
            }};
        }
        load_file!(health_fading);
        load_file!(health_low);
        load_file!(health);
        load_file!(shield_fading);
        load_file!(shield);
        let _ = idx;

        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("1234"),
                size: Extent3d {
                    width: bar_dims.0,
                    height: bar_dims.1,
                    depth_or_array_layers: Self::LAYER_COUNT,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[TextureFormat::Rgba8Unorm],
            },
            TextureDataOrder::LayerMajor,
            &data,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label,
            format: Some(TextureFormat::Rgba8Unorm),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label,
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });

        (bind_group, Vec2::new(bar_dims.0 as f32, bar_dims.1 as f32))
            .wrap_ok()
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    last_inner_size: [u32; 2],

    health_bar_assets: HealthBarAssetsPath,
    bar_start: f32,
    bar_end: f32,
    bar_fading_accel: f32,
    bar_health_low_threshold: f32,
    dmg_tap: f32,
    dmg_tap_margin: f32,
    dmg_held_health: f32,
    dmg_held_health_interval: f32,
    dmg_held_shield_delay: f32,
    dmg_held_shield_threshold: f32,
    dmg_held_shield2health_delay: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_inner_size: [680, 620],
            health_bar_assets: HealthBarAssetsPath::default(),
            bar_start: 38. / 680.,
            bar_end: 1. - 38. / 680.,
            bar_fading_accel: 0.4,
            bar_health_low_threshold: 0.2,
            dmg_tap: 0.2,
            dmg_tap_margin: 0.1,
            dmg_held_health: 0.1,
            dmg_held_health_interval: 0.5,
            dmg_held_shield_delay: 0.5,
            dmg_held_shield_threshold: 0.1,
            dmg_held_shield2health_delay: 2.,
        }
    }
}

impl Config {
    fn config_dir() -> PathBuf {
        std::env::home_dir()
            .unwrap()
            .join(".config")
            .join("live_health_bar")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    fn load() -> Self {
        match std::fs::read_to_string(Self::config_path()) {
            Ok(cfg) => serde_json::from_str(&cfg).unwrap(),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                Self::default()
            }
            Err(err) => {
                panic!("failed to read config: {err:?}");
            }
        }
    }

    fn save(&self) {
        let cfg = serde_json::to_string_pretty(self).unwrap();
        std::fs::create_dir_all(Self::config_dir()).unwrap();
        std::fs::write(Self::config_path(), cfg).unwrap();
    }
}

enum HeldState {
    None,
    Shield { start: Instant, applied: bool },
    Dmg { start: Instant, applied: bool },
}

struct HealthState {
    held_state: HeldState,

    health_fading: f32,
    health_fade_vel: f32,
    health: f32,
    shield_fading: f32,
    shield_fade_vel: f32,
    shield: f32,
}

impl Default for HealthState {
    fn default() -> Self {
        Self {
            held_state: HeldState::None,

            health_fading: 1.,
            health_fade_vel: 0.,
            health: 1.,
            shield_fading: 1.,
            shield_fade_vel: 0.,
            shield: 1.,
        }
    }
}

impl HealthState {
    fn should_update(&self) -> bool {
        self.health_fading != self.health
            || self.shield_fading != self.shield
    }

    fn update(&mut self, dt: f32, config: &Config) {
        let accel = dt * config.bar_fading_accel;

        if self.health_fading > self.health {
            self.health_fade_vel += accel;
            self.health_fading -= self.health_fade_vel * dt;
        }
        if self.health_fading <= self.health {
            self.health_fade_vel = 0.;
            self.health_fading = self.health;
        }

        if self.shield_fading > self.shield {
            self.shield_fade_vel += accel;
            self.shield_fading -= self.shield_fade_vel * dt;
        }
        if self.shield_fading <= self.shield {
            self.shield_fade_vel = 0.;
            self.shield_fading = self.shield;
        }
    }

    fn to_layers_x(
        &self,
        config: &Config,
    ) -> [f32; HealthBarAssetsPath::LAYER_COUNT as usize] {
        let r = |x: f32| {
            (config.bar_end - config.bar_start) * x + config.bar_start
        };

        let health_fading = r(self.health_fading);
        let (health_low, health) =
            if self.health < config.bar_health_low_threshold {
                (r(self.health), 0.)
            } else {
                (0., r(self.health))
            };
        let shield_fading = r(self.shield_fading);
        let shield = r(self.shield);
        [1., health_fading, health_low, health, shield_fading, shield]
    }

    fn apply_hit(&mut self, config: &Config) {
        let dmg = config.dmg_tap
            + rand::random_range(0.0..config.dmg_tap_margin)
            - config.dmg_tap_margin / 2.;
        self.apply_dmg(dmg);
    }

    fn apply_dmg(&mut self, dmg: f32) {
        self.shield -= dmg;
        if self.shield < 0. {
            self.health += self.shield;
            self.shield = 0.;
        }
        if self.health < 0. {
            self.health = 0.;
        }
    }

    /// # Returns
    /// true: need repaint
    fn handle_input(&mut self, pressed: bool, config: &Config) -> bool {
        match &mut self.held_state {
            HeldState::None => {
                if pressed {
                    if self.shield > config.dmg_held_shield_threshold {
                        self.held_state = HeldState::Shield {
                            start: Instant::now(),
                            applied: false,
                        };
                    } else {
                        self.held_state = HeldState::Dmg {
                            start: Instant::now(),
                            applied: false,
                        };
                    }
                } else {
                    return false;
                }
            }
            HeldState::Shield { start, applied } => {
                let elapsed = start.elapsed().as_secs_f32();
                if !*applied && elapsed > config.dmg_held_shield_delay {
                    self.shield = 0.;
                    *start = Instant::now();
                    *applied = true;
                } else if *applied
                    && elapsed > config.dmg_held_shield2health_delay
                {
                    self.held_state = HeldState::Dmg {
                        start: Instant::now(),
                        applied: false,
                    }
                } else if !pressed {
                    if !*applied {
                        self.apply_hit(config);
                    }
                    self.held_state = HeldState::None;
                    return false;
                }
            }
            HeldState::Dmg { start, applied } => {
                let elapsed = start.elapsed().as_secs_f32();
                if elapsed > config.dmg_held_health_interval {
                    *start = Instant::now();
                    *applied = true;
                    self.apply_dmg(config.dmg_held_health);
                } else if !pressed {
                    if !*applied {
                        self.apply_hit(config);
                    }
                    self.held_state = HeldState::None;
                    return false;
                }
            }
        }

        true
    }
}

#[derive(PartialEq)]
enum AssetState {
    Pending,
    Loaded,
    Err(&'static str),
}

struct App {
    config: Config,
    asset_state: AssetState,
    bar_dims: Vec2,
    dims_tx: mpsc::Sender<Result<Vec2, &'static str>>,
    dims_rx: mpsc::Receiver<Result<Vec2, &'static str>>,
    health_state: HealthState,
    key_pressing: bool,
}

impl App {
    fn new(config: Config, cc: &CreationContext) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = &render_state.device;

        let label = Some("bar");

        let shader =
            device.create_shader_module(ShaderModuleDescriptor {
                label,
                source: ShaderSource::Wgsl(
                    include_str!("./bar.wgsl").into(),
                ),
            });

        let bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(
                            SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                ],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label,
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 8,
            });

        let render_pipeline =
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options:
                        PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options:
                        PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: render_state.target_format,
                        blend: Some(
                            BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        ),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let sampler =
            device.create_sampler(&SamplerDescriptor::default());

        let (bind_group, bar_dims) = HealthBarAssetsPath::default()
            .load(
                device,
                &render_state.queue,
                label,
                &bind_group_layout,
                &sampler,
            )
            .unwrap();

        render_state.renderer.write().callback_resources.insert(
            BarRenderResources {
                render_pipeline,
                sampler,
                bind_group_layout,
                bind_group,
            },
        );

        let (dims_tx, dims_rx) = mpsc::channel();
        Self {
            config,
            asset_state: AssetState::Pending,
            bar_dims,
            dims_tx,
            dims_rx,
            health_state: HealthState::default(),
            key_pressing: false,
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, frame: &mut eframe::Frame) {
        while let Ok(dims) = self.dims_rx.try_recv() {
            match dims {
                Ok(dims) => {
                    self.bar_dims = dims;
                }
                Err(msg) => {
                    self.asset_state = AssetState::Err(msg);
                }
            }
        }

        let window = frame.winit_window().unwrap();
        let window_size = window
            .inner_size()
            .to_logical::<u32>(window.scale_factor())
            .into();
        if self.config.last_inner_size != window_size {
            self.config.last_inner_size = window_size;
            self.config.save();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = ui.input(|i| i.stable_dt);
        let key = egui::Key::Space;
        let key_pressed = ui.input(|i| i.key_pressed(key));
        let key_released = ui.input(|i| i.key_released(key));

        if key_pressed {
            self.key_pressing = true;
        }
        if key_released {
            self.key_pressing = false;
        }

        egui::Panel::top(egui::Id::new("bar"))
            .frame(egui::Frame::new())
            .show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    self.bar_dims,
                    egui::Sense::empty(),
                );

                let load_assets =
                    if self.asset_state == AssetState::Pending {
                        self.asset_state = AssetState::Loaded;
                        Some((
                            self.config.health_bar_assets.clone(),
                            self.dims_tx.clone(),
                        ))
                    } else {
                        None
                    };

                self.health_state.update(dt, &self.config);
                if self.health_state.should_update() {
                    ui.request_repaint();
                }

                ui.painter().add(
                    egui_wgpu::Callback::new_paint_callback(
                        rect,
                        BarRenderCallback {
                            x: self
                                .health_state
                                .to_layers_x(&self.config),
                            load_assets,
                        },
                    ),
                );
            });

        egui::CentralPanel::default().show(ui, |ui| {
            if self
                .health_state
                .handle_input(self.key_pressing, &self.config)
            {
                ui.request_repaint();
            }

            macro_rules! cfg_asset {
                ($name:ident) => {
                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .add_filter("PNG", &["png"])
                                .pick_file()
                        {
                            self.config.health_bar_assets.$name =
                                Some(path);
                            self.config.save();
                            self.asset_state = AssetState::Pending;
                        }

                        if ui.button("Clear").clicked()
                            && self
                                .config
                                .health_bar_assets
                                .$name
                                .is_some()
                        {
                            self.config.health_bar_assets.$name = None;
                            self.config.save();
                            self.asset_state = AssetState::Pending;
                        }

                        ui.label(stringify!($name));
                    });
                    ui.label(format!(
                        "{:?}",
                        self.config.health_bar_assets.$name
                    ));
                };
            }
            cfg_asset!(health_empty);
            cfg_asset!(health_fading);
            cfg_asset!(health_low);
            cfg_asset!(health);
            cfg_asset!(shield_fading);
            cfg_asset!(shield);

            ui.separator();

            macro_rules! cfg_slider {
                ($name:ident, $range:expr) => {
                    if ui
                        .add(
                            egui::Slider::new(
                                &mut self.config.$name,
                                $range,
                            )
                            .text(stringify!($name)),
                        )
                        .changed()
                    {
                        self.config.save();
                    }
                };
            }
            cfg_slider!(bar_start, 0.0..=self.config.bar_end);
            cfg_slider!(bar_end, self.config.bar_start..=1.);
            cfg_slider!(bar_fading_accel, 0.0..=5.);
            cfg_slider!(bar_health_low_threshold, 0.0..=1.);
            cfg_slider!(dmg_tap, 0.0..=2.);
            cfg_slider!(dmg_tap_margin, 0.0..=2.);
            cfg_slider!(dmg_held_health, 0.0..=2.);
            cfg_slider!(dmg_held_health_interval, 0.0..=5.);
            cfg_slider!(dmg_held_shield_delay, 0.0..=30.);
            cfg_slider!(dmg_held_shield_threshold, 0.0..=1.);
            cfg_slider!(dmg_held_shield2health_delay, 0.0..=30.);

            if ui.button("Reset Params").clicked() {
                let assets = self.config.health_bar_assets.clone();
                self.config = Config::default();
                self.config.health_bar_assets = assets;
                self.config.save();
            }

            ui.separator();

            ui.horizontal(|ui| {
                if let AssetState::Err(msg) = self.asset_state {
                    ui.label(
                        egui::RichText::new(msg).color(Color32::RED),
                    );
                    ui.separator();
                }

                if ui.button("Reset").clicked() {
                    self.health_state = Default::default();
                }
            });
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}

struct BarRenderResources {
    render_pipeline: RenderPipeline,
    sampler: Sampler,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

struct BarRenderCallback {
    x: [f32; HealthBarAssetsPath::LAYER_COUNT as usize],
    load_assets: Option<(
        HealthBarAssetsPath,
        mpsc::Sender<Result<Vec2, &'static str>>,
    )>,
}

impl egui_wgpu::CallbackTrait for BarRenderCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        if let Some((ref assets, ref dims_tx)) = self.load_assets {
            let res: &mut BarRenderResources =
                callback_resources.get_mut().unwrap();
            match assets.load(
                device,
                queue,
                Some("bar"),
                &res.bind_group_layout,
                &res.sampler,
            ) {
                Ok((bind_group, dims)) => {
                    res.bind_group = bind_group;
                    let _ = dims_tx.send(Ok(dims));
                }
                Err(msg) => {
                    let _ = dims_tx.send(Err(msg));
                }
            };
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let res: &BarRenderResources = callback_resources.get().unwrap();
        render_pass.set_pipeline(&res.render_pipeline);
        render_pass.set_bind_group(0, &res.bind_group, &[]);

        for idx in 0..HealthBarAssetsPath::LAYER_COUNT {
            render_pass.set_immediates(
                0,
                cast_slice(&[Param {
                    index: idx,
                    x: self.x[idx as usize],
                }]),
            );
            render_pass.draw(0..6, 0..1);
        }
    }
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C, packed(1))]
struct Param {
    index: u32,
    x: f32,
}
