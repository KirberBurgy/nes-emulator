use std::sync::Arc;

use winit::event_loop::EventLoop;

use crate::{audio_player::AudioPlayer, cartridge::Cartridge, nes::NES, renderer::Renderer};

pub mod bit_utils;

pub mod memory_bus;

pub mod cpu;
pub mod instructions;

pub mod ppu;
pub mod apu;
pub mod controller;

pub mod mapper;
pub mod mappers;

pub mod cartridge;

pub mod nes;

pub mod renderer;
pub mod audio_player;

struct AppState {
    nes:        NES,
    renderer:   Renderer,
    player:     AudioPlayer,
    dt_accumulator: f32,
    last_render_time: std::time::Instant
}

struct App {
    state: Option<AppState>
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() { return; }

        let cart = Cartridge::load("tests/roms/ppu_vbl_nmi.nes").unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));
        let mut state = AppState { 
            nes:                NES::new(cart),
            player:             AudioPlayer::new(),
            renderer:           Renderer::new(Arc::new(event_loop.create_window(Default::default()).unwrap())),
            dt_accumulator:     0.,
            last_render_time:   std::time::Instant::now()
        };

        state.nes.reset();
        state.player.play();
        state.player.volume = 0.05;

        const N: u32 = 4;

        state.renderer.window.set_min_inner_size(Some(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(256, 240))));
        let _ = state.renderer.window.request_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(256 * N, 240 * N)));

        self.state = Some(state);
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(state) = &mut self.state else {
            return;
        };

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(state.last_render_time).as_secs_f32();

        state.last_render_time = now;
        
        state.dt_accumulator += elapsed;

        if state.dt_accumulator > 0.1 {
            state.dt_accumulator = 0.1;
        }

        let mut needs_redraw = false;

        while state.dt_accumulator >= 0.0166 {
            let target_frame = state.nes.bus.ppu.frame + 1;
            
            while state.nes.bus.ppu.frame < target_frame {
                state.nes.tick();
                state.player.tick(&state.nes.bus.apu);
            }

            state.dt_accumulator -= 0.0166;
            needs_redraw = true;
        }

        if needs_redraw {
            state.renderer.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) 
    {
        let Some(state) = &mut self.state else {
            return;
        };

        if state.renderer.window.id() != window_id { return; }

        match event {
            winit::event::WindowEvent::Resized(size) => {
                state.renderer.config.width = size.width;
                state.renderer.config.height = size.height;

                state.renderer.surface.configure(&state.renderer.device, &state.renderer.config);
            }

            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            winit::event::WindowEvent::RedrawRequested => {
                state.renderer.upload_framebuffer(&state.nes.bus.ppu.framebuffer);
                state.renderer.render();
            },

            winit::event::WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                let winit::keyboard::PhysicalKey::Code(code) = event.physical_key else { 
                    return;
                };

                let on = matches!(event.state, winit::event::ElementState::Pressed);

                match code {
                    winit::keyboard::KeyCode::ArrowLeft     => state.nes.bus.player_1.left      = on,
                    winit::keyboard::KeyCode::ArrowRight    => state.nes.bus.player_1.right     = on,
                    winit::keyboard::KeyCode::ArrowUp       => state.nes.bus.player_1.up        = on,
                    winit::keyboard::KeyCode::ArrowDown     => state.nes.bus.player_1.down      = on,
                    winit::keyboard::KeyCode::Backspace     => state.nes.bus.player_1.select    = on,
                    winit::keyboard::KeyCode::Enter         => state.nes.bus.player_1.start     = on,
                    winit::keyboard::KeyCode::KeyZ          => state.nes.bus.player_1.a         = on,
                    winit::keyboard::KeyCode::KeyX          => state.nes.bus.player_1.b         = on,

                    winit::keyboard::KeyCode::KeyS          => state.nes.bus.cartridge.borrow_mut().save_sram(),

                    _ => {}
                }
            }
            _ => {}
        }
    }
}



fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut App{ state: None }).unwrap();
}